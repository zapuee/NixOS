use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Palette {
    pub colors: BTreeMap<String, String>,
}

impl Palette {
    pub fn get(&self, role: &str) -> Result<&str> {
        self.colors
            .get(role)
            .map(String::as_str)
            .ok_or_else(|| anyhow::anyhow!("palette does not provide color role '{role}'"))
    }

    pub fn validate_color(value: &str) -> Result<()> {
        let hex = value.strip_prefix('#').unwrap_or(value);
        if !matches!(hex.len(), 6 | 8) || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            bail!("unsupported color '{value}'; expected #RRGGBB or #RRGGBBAA");
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        for (role, value) in &self.colors {
            Self::validate_color(value)
                .map_err(|err| anyhow::anyhow!("invalid color for '{role}': {err}"))?;
        }
        Ok(())
    }
}
