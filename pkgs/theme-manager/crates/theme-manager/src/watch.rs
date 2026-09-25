use crate::{
    health::{self, WatchHealth},
    runtime::{Runtime, parent_or_current},
};
use anyhow::{Context, Result};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, RecvTimeoutError},
    thread,
    time::Duration,
};
use theme_core::paths::xdg_config_home;
use tracing::{error, info, warn};

fn nearest_existing(path: &Path) -> Option<PathBuf> {
    let mut current = path;
    loop {
        if current.exists() {
            return Some(current.to_path_buf());
        }
        current = current.parent()?;
    }
}

fn watch_location(path: &Path) -> PathBuf {
    if path.is_dir() {
        path.to_path_buf()
    } else {
        parent_or_current(path)
    }
}

fn reconcile(
    watcher: &mut RecommendedWatcher,
    watched: &mut BTreeSet<PathBuf>,
    requested: Vec<PathBuf>,
) {
    let desired = requested
        .into_iter()
        .map(|path| watch_location(&path))
        .filter_map(|path| nearest_existing(&path))
        .collect::<BTreeSet<_>>();
    for path in watched.difference(&desired).cloned().collect::<Vec<_>>() {
        if watcher.unwatch(&path).is_ok() {
            watched.remove(&path);
        }
    }
    for path in desired.difference(watched).cloned().collect::<Vec<_>>() {
        match watcher.watch(&path, RecursiveMode::NonRecursive) {
            Ok(()) => {
                info!(path = %path.display(), "watching");
                watched.insert(path);
            }
            Err(error) => warn!(path = %path.display(), %error, "failed to watch path"),
        }
    }
}

fn is_change(event: &Event, health_path: &Path, relevant: &BTreeSet<PathBuf>) -> bool {
    matches!(
        event.kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    ) && event.paths.iter().any(|path| {
        path != health_path
            && !(path.parent() == health_path.parent()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with(".watch-status.json.")))
            && relevant.iter().any(|input| {
                path == input || input.is_dir() && path.parent() == Some(input.as_path())
            })
    })
}

fn wait_for_change(
    receiver: &Receiver<notify::Result<Event>>,
    timeout: Duration,
    health_path: &Path,
    relevant: &BTreeSet<PathBuf>,
) -> Result<bool> {
    loop {
        match receiver.recv_timeout(timeout) {
            Ok(Ok(event)) if is_change(&event, health_path, relevant) => return Ok(true),
            Ok(Ok(_)) => {}
            Ok(Err(error)) => warn!(%error, "filesystem watch error"),
            Err(RecvTimeoutError::Timeout) => return Ok(false),
            Err(RecvTimeoutError::Disconnected) => {
                anyhow::bail!("filesystem watcher channel closed")
            }
        }
    }
}

pub fn run(config_path: Option<PathBuf>) -> Result<()> {
    let (sender, receiver) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |event| {
        let _ = sender.send(event);
    })
    .context("failed to create filesystem watcher")?;
    let mut watched = BTreeSet::new();
    let initial_config = config_path
        .clone()
        .unwrap_or(xdg_config_home()?.join("theme-manager/config.toml"));
    let initial_config_dir = parent_or_current(&initial_config);
    reconcile(&mut watcher, &mut watched, vec![initial_config_dir.clone()]);
    let mut relevant = BTreeSet::from([initial_config]);

    let fallback_state = theme_core::paths::xdg_state_home()?.join("theme-manager/state.toml");
    let mut health_path = health::path_for_state(&fallback_state);
    let mut status = WatchHealth::starting();
    let retries = [
        Duration::from_millis(250),
        Duration::from_secs(1),
        Duration::from_secs(4),
        Duration::from_secs(30),
    ];
    let mut failures = 0usize;

    loop {
        status.last_attempt_ms = health::now_ms();
        match Runtime::load(config_path.as_deref()) {
            Ok(runtime) => {
                health_path = health::path_for_state(&runtime.config.state_file()?);
                let mut paths = runtime.watch_paths()?;
                paths.push(runtime.config_path.clone());
                relevant = paths.iter().cloned().collect();
                reconcile(&mut watcher, &mut watched, paths);
                match runtime.active_theme()? {
                    Some(theme) => match runtime.apply(&theme, false) {
                        Ok(report) => {
                            let changed = report
                                .wires
                                .iter()
                                .map(|wire| wire.changed.len())
                                .sum::<usize>();
                            info!(theme, changed, "applied theme");
                            status.last_success_ms = Some(health::now_ms());
                            status.active_theme = Some(theme);
                            status.last_error = None;
                            failures = 0;
                        }
                        Err(error) => {
                            status.last_error = Some(format!("{error:#}"));
                            failures = failures.saturating_add(1);
                            error!(%error, "apply failed; watcher remains active");
                        }
                    },
                    None => {
                        status.last_error = Some("no active/default theme".into());
                        warn!("no active/default theme; watcher is waiting");
                    }
                }
            }
            Err(error) => {
                status.last_error = Some(format!("{error:#}"));
                failures = failures.saturating_add(1);
                error!(%error, "configuration is invalid; watcher remains active");
            }
        }
        if let Err(error) = health::write(&health_path, &status) {
            warn!(%error, "failed to write watcher status");
        }
        let timeout = if failures == 0 {
            Duration::from_secs(30)
        } else {
            retries[failures.saturating_sub(1).min(retries.len() - 1)]
        };
        if wait_for_change(&receiver, timeout, &health_path, &relevant)? {
            thread::sleep(Duration::from_millis(80));
            while receiver.try_recv().is_ok() {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::is_change;
    use notify::{Event, EventKind, event::ModifyKind};
    use std::{collections::BTreeSet, path::PathBuf};

    #[test]
    fn parent_watch_filters_unrelated_siblings() {
        let palette = PathBuf::from("/config/theme-manager/palette.json");
        let relevant = BTreeSet::from([palette.clone()]);
        let health = PathBuf::from("/state/theme-manager/watch-status.json");
        let palette_event =
            Event::new(EventKind::Modify(ModifyKind::Any)).add_path(palette.clone());
        let sibling_event = Event::new(EventKind::Modify(ModifyKind::Any))
            .add_path(palette.with_file_name("unrelated.json"));

        assert!(is_change(&palette_event, &health, &relevant));
        assert!(!is_change(&sibling_event, &health, &relevant));
    }
}
