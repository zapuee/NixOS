use crate::fs::write_if_changed;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    path::Path,
};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct State {
    #[serde(default)]
    pub active_profile: Option<String>,

    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub disabled_targets: BTreeSet<String>,

    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub generated_outputs: BTreeMap<String, GeneratedOutput>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GeneratedOutput {
    pub target: String,
    pub hash: String,
}

pub fn content_hash(contents: impl AsRef<[u8]>) -> String {
    blake3::hash(contents.as_ref()).to_hex().to_string()
}

pub fn read(path: &Path) -> Result<State> {
    if !path.exists() {
        return Ok(State::default());
    }
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read state {}", path.display()))?;
    toml::from_str(&raw).with_context(|| format!("failed to parse state {}", path.display()))
}

pub fn write(path: &Path, state: &State) -> Result<bool> {
    let raw = toml::to_string_pretty(state).context("failed to serialize state")?;
    write_if_changed(path, &raw)
}

pub fn update(path: &Path, mutate: impl FnOnce(&mut State)) -> Result<bool> {
    with_lock(path, || {
        let mut state = read(path)?;
        mutate(&mut state);
        write(path, &state)
    })
}

pub fn with_lock<T>(path: &Path, operation: impl FnOnce() -> Result<T>) -> Result<T> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .with_context(|| format!("failed to create state directory {}", parent.display()))?;

    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .context("state file has no valid file name")?;
    let lock_path = path.with_file_name(format!(".{file_name}.lock"));
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&lock_path)
        .with_context(|| format!("failed to open state lock {}", lock_path.display()))?;

    fs2::FileExt::lock_exclusive(&lock)
        .with_context(|| format!("failed to lock state {}", path.display()))?;
    operation()
}

#[cfg(test)]
mod tests {
    use super::{read, update};
    use std::{
        fs,
        sync::{Arc, Barrier},
        thread,
    };

    #[test]
    fn concurrent_updates_preserve_every_field_change() {
        let root =
            std::env::temp_dir().join(format!("theme-manager-state-test-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let path = Arc::new(root.join("state.toml"));
        let barrier = Arc::new(Barrier::new(8));
        let threads = (0..8)
            .map(|index| {
                let path = Arc::clone(&path);
                let barrier = Arc::clone(&barrier);
                thread::spawn(move || {
                    barrier.wait();
                    update(&path, |state| {
                        state.disabled_targets.insert(format!("target-{index}"));
                    })
                    .unwrap();
                })
            })
            .collect::<Vec<_>>();

        for worker in threads {
            worker.join().unwrap();
        }

        let state = read(&path).unwrap();
        assert_eq!(state.disabled_targets.len(), 8);
        let _ = fs::remove_dir_all(root);
    }
}
