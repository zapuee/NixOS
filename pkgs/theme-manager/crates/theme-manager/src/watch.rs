use crate::runtime::{Runtime, parent_or_current};
use anyhow::{Context, Result};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver},
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

fn register_paths(
    watcher: &mut RecommendedWatcher,
    watched: &mut BTreeSet<PathBuf>,
    paths: Vec<PathBuf>,
) {
    for requested in paths {
        let Some(path) = watch_missing_parent(&requested) else {
            continue;
        };

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

fn wait_for_change(rx: &Receiver<notify::Result<Event>>) -> Result<()> {
    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                if is_change_event(&event) {
                    return Ok(());
                }
            }

            Ok(Err(err)) => {
                warn!(%err, "filesystem watch error");
            }

            Err(err) => {
                return Err(err).context("filesystem watcher channel closed");
            }
        }
    }
}

fn apply_active(runtime: &Runtime) -> Result<()> {
    let Some(profile) = runtime.engine.active_profile()? else {
        warn!("no active/default profile; waiting for `theme-manager apply <profile>`");

        return Ok(());
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

    Ok(())
}

pub fn run(config_path: Option<PathBuf>) -> Result<()> {
    let (tx, rx) = mpsc::channel();

    let mut watcher = notify::recommended_watcher(move |event: notify::Result<Event>| {
        let _ = tx.send(event);
    })
    .context("failed to create filesystem watcher")?;

    let mut watched = BTreeSet::new();

    let initial_config_dir = match &config_path {
        Some(path) => parent_or_current(path),
        None => xdg_config_home()?.join("theme-manager"),
    };
    register_paths(&mut watcher, &mut watched, vec![initial_config_dir]);

    loop {
        let runtime = match Runtime::load(config_path.as_deref()) {
            Ok(runtime) => runtime,

            Err(err) => {
                error!(
                    %err,
                    "runtime configuration is invalid; waiting for another file change"
                );

                wait_for_change(&rx)?;
                continue;
            }
        };

        register_paths(&mut watcher, &mut watched, runtime.watch_paths()?);

        if let Err(err) = apply_active(&runtime) {
            error!(
                %err,
                "apply failed; watcher remains active"
            );
        }

        // Ignore read/access events and wait for an actual filesystem
        // modification.
        wait_for_change(&rx)?;

        // Editors and template generators often produce multiple events for
        // one logical change. Give them a moment to finish, then collapse the
        // batch into one apply.
        thread::sleep(Duration::from_millis(80));

        while rx.try_recv().is_ok() {}
    }
}
