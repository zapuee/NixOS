use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

fn default_schema() -> u32 {
    1
}
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Numeric {
    Integer(i64),
    Float(f64),
}

impl Numeric {
    pub fn as_f64(self) -> f64 {
        match self {
            Self::Integer(value) => value as f64,
            Self::Float(value) => value,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum NumberRef {
    Integer(i64),
    Float(f64),
    Ref(String),
}

impl NumberRef {
    pub fn literal(&self) -> Option<f64> {
        match self {
            Self::Integer(value) => Some(*value as f64),
            Self::Float(value) => Some(*value),
            Self::Ref(_) => None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    #[serde(default = "default_schema")]
    pub schema: u32,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub variables: Variables,
    pub layout: Layout,
    pub focus_ring: FocusRing,
    pub shadow: Shadow,
    #[serde(default)]
    pub roles: BTreeMap<String, RoleStyle>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Variables {
    #[serde(default)]
    pub opacity: BTreeMap<String, Numeric>,
    #[serde(default)]
    pub radius: BTreeMap<String, Numeric>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Layout {
    pub gaps: Numeric,
    pub outer_gaps: Numeric,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FocusRing {
    pub width: Numeric,
    pub active_opacity: Numeric,
    pub inactive_opacity: Numeric,
    pub urgent_opacity: Numeric,
    pub active_color: String,
    pub inactive_color: String,
    pub urgent_color: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Shadow {
    pub strength: Numeric,
    pub color: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RoleStyle {
    #[serde(default)]
    pub background_opacity: Option<NumberRef>,
    #[serde(default)]
    pub blur: Option<bool>,
    #[serde(default)]
    pub corner_radius: Option<NumberRef>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedTheme {
    pub schema: u32,
    pub profile: String,
    pub description: String,
    pub layout: ResolvedLayout,
    pub focus_ring: ResolvedFocusRing,
    pub shadow: ResolvedShadow,
    pub roles: BTreeMap<String, ResolvedRole>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedLayout {
    pub gaps: f64,
    pub outer_gaps: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedFocusRing {
    pub width: f64,
    pub active_opacity: f64,
    pub inactive_opacity: f64,
    pub urgent_opacity: f64,
    pub active_color: String,
    pub inactive_color: String,
    pub urgent_color: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedShadow {
    pub strength: f64,
    pub color: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedRole {
    pub background_opacity: Option<f64>,
    pub blur: Option<bool>,
    pub corner_radius: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::RoleStyle;

    #[test]
    fn unknown_role_options_are_rejected() {
        let error = toml::from_str::<RoleStyle>("backgroun_opacity = 0.5")
            .expect_err("misspelled role options must not be silently ignored");
        assert!(error.to_string().contains("unknown field"));
    }
}
