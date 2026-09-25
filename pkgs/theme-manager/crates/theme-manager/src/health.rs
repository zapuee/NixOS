use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchHealth {
    pub schema: u32,
    pub pid: u32,
    pub started_ms: u64,
    pub last_attempt_ms: u64,
    pub last_success_ms: Option<u64>,
    pub active_theme: Option<String>,
    pub last_error: Option<String>,
}

impl WatchHealth {
    pub fn starting() -> Self {
        let now = now_ms();
        Self {
            schema: 1,
            pid: std::process::id(),
            started_ms: now,
            last_attempt_ms: now,
            last_success_ms: None,
            active_theme: None,
            last_error: None,
        }
    }
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

pub fn path_for_state(state_file: &Path) -> PathBuf {
    state_file.with_file_name("watch-status.json")
}

pub fn write(path: &Path, health: &WatchHealth) -> Result<()> {
    let raw = serde_json::to_string_pretty(health).context("failed to serialize watcher health")?;
    theme_core::fs::write_if_changed(path, &format!("{raw}\n"))?;
    Ok(())
}

pub fn read(path: &Path) -> Result<Option<WatchHealth>> {
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(error)
                .with_context(|| format!("failed to read watcher health {}", path.display()));
        }
    };
    let health = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse watcher health {}", path.display()))?;
    Ok(Some(health))
}
