use crate::fs::write_if_changed;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    path::Path,
};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    #[serde(default)]
    pub active_theme: Option<String>,
}

pub fn read(path: &Path) -> Result<State> {
    if !path.exists() {
        return Ok(State::default());
    }
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read state {}", path.display()))?;
    match toml::from_str(&raw) {
        Ok(state) => Ok(state),
        Err(current_error) => migrate_v0_2(&raw)
            .with_context(|| format!("failed to parse state {}: {current_error}", path.display())),
    }
}

fn migrate_v0_2(raw: &str) -> Result<State> {
    let value: toml::Value = toml::from_str(raw)?;
    let table = value
        .as_table()
        .context("legacy state must be a TOML table")?;
    const LEGACY_KEYS: &[&str] = &[
        "active_profile",
        "disabled_targets",
        "generated_outputs",
        "last_transaction",
    ];
    if let Some(unknown) = table
        .keys()
        .find(|key| !LEGACY_KEYS.contains(&key.as_str()))
    {
        anyhow::bail!("unknown state field '{unknown}'");
    }
    let active_theme = table
        .get("active_profile")
        .map(|value| {
            value
                .as_str()
                .map(str::to_string)
                .context("legacy active_profile must be a string")
        })
        .transpose()?;
    Ok(State { active_theme })
}

pub fn write(path: &Path, state: &State) -> Result<bool> {
    let raw = toml::to_string_pretty(state).context("failed to serialize state")?;
    write_if_changed(path, &raw)
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
    use super::migrate_v0_2;

    #[test]
    fn migrates_only_known_v0_2_state_fields() {
        let state = migrate_v0_2(
            r#"active_profile = "glass"
disabled_targets = ["foot"]
[generated_outputs]
"#,
        )
        .unwrap();
        assert_eq!(state.active_theme.as_deref(), Some("glass"));
        assert!(migrate_v0_2("active_profiel = \"glass\"").is_err());
    }
}
