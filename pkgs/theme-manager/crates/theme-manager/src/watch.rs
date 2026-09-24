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

fn watch_missing_parent(path: &Path) -> Option<PathBuf> {
    let mut current = path;

    loop {
        if current.exists() {
            return Some(current.to_path_buf());
        }

        current = current.parent()?;
    }
}

fn reconcile_paths(
    watcher: &mut RecommendedWatcher,
    watched: &mut BTreeSet<PathBuf>,
    paths: Vec<PathBuf>,
) {
    let desired = paths
        .into_iter()
        .filter_map(|requested| watch_missing_parent(&requested))
        .collect::<BTreeSet<_>>();

    for path in watched.difference(&desired).cloned().collect::<Vec<_>>() {
        if let Err(err) = watcher.unwatch(&path) {
            warn!(path = %path.display(), %err, "failed to stop watching path");
        } else {
            watched.remove(&path);
        }
    }

    for path in desired {
        if watched.contains(&path) {
            continue;
        }

        match watcher.watch(&path, RecursiveMode::NonRecursive) {
            Ok(()) => {
                info!(path = %path.display(), "watching");
                watched.insert(path);
            }

            Err(err) => {
                warn!(
                    path = %path.display(),
                    %err,
                    "failed to watch path"
                );
            }
        }
    }
}

fn is_change_event(event: &Event) -> bool {
    matches!(
        event.kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    )
}

fn is_health_artifact(path: &Path, health_path: &Path) -> bool {
    if path == health_path {
        return true;
    }
    path.parent() == health_path.parent()
        && path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with(".watch-status.json.") && name.ends_with(".tmp"))
}

fn wait_for_signal(
    rx: &Receiver<notify::Result<Event>>,
    timeout: Duration,
    ignored_path: &Path,
) -> Result<bool> {
    loop {
        match rx.recv_timeout(timeout) {
            Ok(Ok(event)) => {
                if is_change_event(&event)
                    && event
                        .paths
                        .iter()
                        .any(|path| !is_health_artifact(path, ignored_path))
                {
                    return Ok(true);
                }
            }

            Ok(Err(err)) => {
                warn!(%err, "filesystem watch error");
            }

            Err(RecvTimeoutError::Timeout) => return Ok(false),
            Err(RecvTimeoutError::Disconnected) => {
                anyhow::bail!("filesystem watcher channel closed")
            }
        }
    }
}

fn apply_active(runtime: &Runtime) -> Result<Option<String>> {
    let Some(profile) = runtime.engine.active_profile()? else {
        warn!("no active/default profile; waiting for `theme-manager apply <profile>`");

        return Ok(None);
    };

    let report = runtime.engine.reapply(&profile)?;

    let changed = report
        .targets
        .iter()
        .map(|target| target.changed.len())
        .sum::<usize>();

    info!(
        profile = %profile,
        changed,
        "applied profile"
    );

    Ok(Some(profile))
}

pub fn run(config_path: Option<PathBuf>) -> Result<()> {
    let (tx, rx) = mpsc::channel();

    let mut watcher = notify::recommended_watcher(move |event: notify::Result<Event>| {
        let _ = tx.send(event);
    })
    .context("failed to create filesystem watcher")?;

    let mut watched = BTreeSet::new();
    let mut health_status = WatchHealth::starting();
    let fallback_state = theme_core::paths::xdg_state_home()?.join("theme-manager/state.toml");
    let mut health_path = health::path_for_state(&fallback_state);
    if let Err(error) = health::write(&health_path, &health_status) {
        warn!(%error, "failed to write watcher health");
    }

    let initial_config_dir = match &config_path {
        Some(path) => parent_or_current(path),
        None => xdg_config_home()?.join("theme-manager"),
    };
    reconcile_paths(&mut watcher, &mut watched, vec![initial_config_dir.clone()]);

    let retry_delays = [
        Duration::from_millis(250),
        Duration::from_secs(1),
        Duration::from_secs(4),
    ];
    let mut consecutive_failures = 0usize;

    loop {
        health_status.last_attempt_ms = health::now_ms();
        let runtime = match Runtime::load(config_path.as_deref()) {
            Ok(runtime) => runtime,

            Err(err) => {
                health_status.last_error = Some(format!("{err:#}"));
                if let Err(health_error) = health::write(&health_path, &health_status) {
                    warn!(%health_error, "failed to write watcher health");
                }
                error!(
                    %err,
                    "runtime configuration is invalid; waiting to retry"
                );
                let delay = retry_delays
                    .get(consecutive_failures)
                    .copied()
                    .unwrap_or(Duration::from_secs(30));
                consecutive_failures = consecutive_failures.saturating_add(1);
                let changed = wait_for_signal(&rx, delay, &health_path)?;
                if changed {
                    thread::sleep(Duration::from_millis(80));
                    while rx.try_recv().is_ok() {}
                }
                continue;
            }
        };

        health_path = health::path_for_state(&runtime.engine.state_file()?);
        let mut desired = runtime.watch_paths()?;
        desired.push(initial_config_dir.clone());
        reconcile_paths(&mut watcher, &mut watched, desired);

        match apply_active(&runtime) {
            Ok(profile) => {
                health_status.last_success_ms = Some(health::now_ms());
                health_status.active_profile = profile;
                health_status.last_error = None;
                consecutive_failures = 0;
            }
            Err(err) => {
                health_status.last_error = Some(format!("{err:#}"));
                consecutive_failures = consecutive_failures.saturating_add(1);
                error!(%err, "apply failed; watcher remains active");
            }
        }
        if let Err(error) = health::write(&health_path, &health_status) {
            warn!(%error, "failed to write watcher health");
        }

        let timeout = if consecutive_failures == 0 {
            Duration::from_secs(30)
        } else {
            retry_delays
                .get(consecutive_failures.saturating_sub(1))
                .copied()
                .unwrap_or(Duration::from_secs(30))
        };
        let changed = wait_for_signal(&rx, timeout, &health_path)?;

        if changed {
            // Editors and template generators often produce multiple events for
            // one logical change. Give them a moment to finish, then collapse the
            // batch into one apply.
            thread::sleep(Duration::from_millis(80));
            while rx.try_recv().is_ok() {}
        }
    }
}
