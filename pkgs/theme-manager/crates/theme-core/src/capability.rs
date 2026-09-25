use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    Colors,
    Font,
    CursorInstallation,
    CursorSelection,
    BackgroundAlpha,
    Reload,
    CursorColor,
    SelectionColor,
    Padding,
    Csd,
    Scrollback,
    NativeMessaging,
    ApplicationStructure,
    WindowStructure,
    Gaps,
    FocusRing,
    Shadow,
    CornerRadius,
}

impl fmt::Display for Capability {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = serde_json::to_value(self).map_err(|_| fmt::Error)?;
        formatter.write_str(value.as_str().ok_or(fmt::Error)?)
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Cadence {
    Runtime,
    Session,
    Rebuild,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CapabilityClaim {
    pub application: String,
    pub capability: Capability,
    pub owner: String,
    pub source: String,
    pub cadence: Cadence,
    pub boundary: String,
    #[serde(default = "provided")]
    pub provides: bool,
}

fn provided() -> bool {
    true
}

#[derive(Debug, Clone, Serialize)]
pub struct CapabilityGap {
    pub application: String,
    pub capability: Capability,
}

#[derive(Debug, Clone, Serialize)]
pub struct CapabilityPlan {
    pub complete: bool,
    pub claims: Vec<CapabilityClaim>,
    pub optional_gaps: Vec<CapabilityGap>,
    pub supported_but_unselected: Vec<CapabilityClaim>,
    pub errors: Vec<String>,
}

pub fn build_plan(
    contracts: &BTreeMap<String, crate::config::ApplicationContract>,
    claims: Vec<CapabilityClaim>,
) -> CapabilityPlan {
    let mut errors = Vec::new();
    let mut optional_gaps = Vec::new();
    let mut selected = claims
        .iter()
        .filter(|claim| claim.provides)
        .cloned()
        .collect::<Vec<_>>();
    let mut supported = claims
        .iter()
        .filter(|claim| !claim.provides)
        .cloned()
        .collect::<Vec<_>>();
    selected.sort_by(claim_order);
    supported.sort_by(claim_order);

    let mut owners: BTreeMap<(String, Capability), BTreeSet<String>> = BTreeMap::new();
    let mut boundaries: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for claim in &selected {
        owners
            .entry((claim.application.clone(), claim.capability))
            .or_default()
            .insert(claim.owner.clone());
        boundaries
            .entry(claim.boundary.clone())
            .or_default()
            .insert(claim.owner.clone());
    }

    for ((application, capability), claim_owners) in &owners {
        if claim_owners.len() > 1 {
            errors.push(format!(
                "application '{application}' capability '{capability}' has multiple owners: {}",
                claim_owners.iter().cloned().collect::<Vec<_>>().join(", ")
            ));
        }
    }
    for (boundary, boundary_owners) in &boundaries {
        if boundary_owners.len() > 1 {
            errors.push(format!(
                "boundary '{boundary}' has multiple owners: {}",
                boundary_owners
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }

    for (application, contract) in contracts {
        for capability in &contract.required {
            if !owners.contains_key(&(application.clone(), *capability)) {
                errors.push(format!(
                    "application '{application}' is missing required capability '{capability}'"
                ));
            }
        }
        for capability in &contract.optional {
            if !owners.contains_key(&(application.clone(), *capability)) {
                optional_gaps.push(CapabilityGap {
                    application: application.clone(),
                    capability: *capability,
                });
            }
        }
    }

    CapabilityPlan {
        complete: errors.is_empty(),
        claims: selected,
        optional_gaps,
        supported_but_unselected: supported,
        errors,
    }
}

impl CapabilityPlan {
    pub fn require_complete(&self) -> Result<()> {
        if !self.complete {
            bail!("capability plan is invalid:\n{}", self.errors.join("\n"));
        }
        Ok(())
    }
}

fn claim_order(left: &CapabilityClaim, right: &CapabilityClaim) -> std::cmp::Ordering {
    (&left.application, left.capability, &left.owner).cmp(&(
        &right.application,
        right.capability,
        &right.owner,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ApplicationContract;

    fn claim(capability: Capability, owner: &str) -> CapabilityClaim {
        CapabilityClaim {
            application: "foot".into(),
            capability,
            owner: owner.into(),
            source: owner.into(),
            cadence: Cadence::Runtime,
            boundary: format!("foot:{capability}"),
            provides: true,
        }
    }

    #[test]
    fn missing_required_fails_and_missing_optional_is_visible() {
        let contracts = BTreeMap::from([(
            "foot".into(),
            ApplicationContract {
                required: BTreeSet::from([Capability::Colors, Capability::BackgroundAlpha]),
                optional: BTreeSet::from([Capability::Reload]),
            },
        )]);
        let plan = build_plan(&contracts, vec![claim(Capability::Colors, "colors")]);
        assert!(!plan.complete);
        assert!(plan.errors[0].contains("background_alpha"));
        assert_eq!(plan.optional_gaps[0].capability, Capability::Reload);
    }

    #[test]
    fn duplicate_capability_owners_fail_even_with_different_files() {
        let plan = build_plan(
            &BTreeMap::new(),
            vec![
                claim(Capability::Colors, "stylix"),
                claim(Capability::Colors, "noctalia"),
            ],
        );
        assert!(!plan.complete);
        assert!(plan.errors[0].contains("multiple owners"));
    }
}
