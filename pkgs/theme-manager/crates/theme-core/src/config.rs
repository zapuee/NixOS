use crate::paths::{default_config_path, expand_path};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

fn default_profiles_dir() -> String {
    "$XDG_CONFIG_HOME/theme-manager/profiles".into()
}

fn default_state_file() -> String {
    "$XDG_STATE_HOME/theme-manager/state.toml".into()
}

fn default_true() -> bool {
    true
}

fn default_memory_mib() -> usize {
    32
}

fn default_execution_ms() -> u64 {
    250
}

fn default_io_mib() -> usize {
    4
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExtensionLimits {
    #[serde(default = "default_memory_mib")]
    pub memory_mib: usize,
    #[serde(default = "default_execution_ms")]
    pub execution_ms: u64,
    #[serde(default = "default_io_mib")]
    pub io_mib: usize,
}

impl Default for ExtensionLimits {
    fn default() -> Self {
        Self {
            memory_mib: default_memory_mib(),
            execution_ms: default_execution_ms(),
            io_mib: default_io_mib(),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExtensionsConfig {
    #[serde(default)]
    pub dirs: Vec<String>,
    #[serde(default)]
    pub limits: ExtensionLimits,
}

impl ExtensionsConfig {
    pub fn validate(&self) -> Result<()> {
        if !(4..=256).contains(&self.limits.memory_mib) {
            anyhow::bail!("extensions.limits.memory_mib must be between 4 and 256");
        }
        if !(10..=5_000).contains(&self.limits.execution_ms) {
            anyhow::bail!("extensions.limits.execution_ms must be between 10 and 5000");
        }
        if !(1..=64).contains(&self.limits.io_mib) {
            anyhow::bail!("extensions.limits.io_mib must be between 1 and 64");
        }
        Ok(())
    }

    pub fn directories(&self) -> Result<Vec<PathBuf>> {
        let mut dirs = Vec::new();
        if let Some(data_dir) =
            std::env::var_os("THEME_MANAGER_DATA_DIR").filter(|value| !value.is_empty())
        {
            dirs.push(PathBuf::from(data_dir).join("plugins"));
        }
        if let Ok(config_home) = crate::paths::xdg_config_home() {
            dirs.push(config_home.join("theme-manager/plugins"));
        }
        for dir in &self.dirs {
            dirs.push(expand_path(dir)?);
        }
        dirs.sort();
        dirs.dedup();
        Ok(dirs)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TuiConfig {
    #[serde(default = "default_true")]
    pub vim_keys: bool,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self { vim_keys: true }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default = "default_profiles_dir")]
    pub profiles_dir: String,

    #[serde(default = "default_state_file")]
    pub state_file: String,

    #[serde(default)]
    pub default_profile: Option<String>,

    #[serde(default)]
    pub tui: TuiConfig,

    #[serde(default)]
    pub extensions: ExtensionsConfig,

    pub provider: ProviderSpec,

    #[serde(default)]
    pub targets: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProviderSpec {
    pub kind: String,

    #[serde(flatten)]
    pub settings: BTreeMap<String, toml::Value>,
}

impl ProviderSpec {
    pub fn settings_value(&self) -> toml::Value {
        toml::Value::Table(self.settings.clone().into_iter().collect())
    }
}

impl Config {
    pub fn load(path: Option<&Path>) -> Result<(Self, PathBuf)> {
        let path = match path {
            Some(path) => path.to_path_buf(),
            None => default_config_path()?,
        };

        let raw = fs::read_to_string(&path)
            .with_context(|| format!("failed to read config {}", path.display()))?;
        let config: Self = toml::from_str(&raw)
            .with_context(|| format!("failed to parse config {}", path.display()))?;
        config.extensions.validate()?;
        Ok((config, path))
    }

    pub fn profiles_dir(&self) -> Result<PathBuf> {
        expand_path(&self.profiles_dir)
    }

    pub fn state_file(&self) -> Result<PathBuf> {
        expand_path(&self.state_file)
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    const BASE_CONFIG: &str = r##"
[provider]
kind = "luau:static"
"##;

    #[test]
    fn vim_keys_default_to_enabled() {
        let config: Config = toml::from_str(BASE_CONFIG).unwrap();
        assert!(config.tui.vim_keys);
    }

    #[test]
    fn vim_keys_can_be_disabled() {
        let config: Config = toml::from_str(
            r##"
[tui]
vim_keys = false

[provider]
kind = "luau:static"
"##,
        )
        .unwrap();
        assert!(!config.tui.vim_keys);
    }

    #[test]
    fn unknown_tui_options_are_rejected() {
        let error = toml::from_str::<Config>(
            r##"
[tui]
vim_keyz = false

[provider]
kind = "luau:static"
"##,
        )
        .unwrap_err();
        assert!(error.to_string().contains("unknown field"));
    }
}
