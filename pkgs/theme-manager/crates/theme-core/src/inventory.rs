use crate::{Cadence, CapabilityClaim, LibrarySelection, ThemeBundle};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::Path};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum LibraryKind {
    Cursor,
    Typography,
    Icon,
    Wallpaper,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LibraryItem {
    pub installed: bool,
    pub effective: bool,
    pub selection_cadence: Cadence,
    #[serde(default)]
    pub required_consumers: Vec<String>,
    #[serde(default)]
    pub consumers: BTreeMap<String, String>,
    #[serde(default)]
    pub values: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AppearanceLibrary {
    pub kind: LibraryKind,
    pub source: String,
    #[serde(default)]
    pub revision: Option<String>,
    pub installation_cadence: Cadence,
    #[serde(default)]
    pub items: BTreeMap<String, LibraryItem>,
}

fn default_schema() -> u32 {
    1
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AppearanceInventory {
    #[serde(default = "default_schema")]
    pub schema: u32,
    pub libraries: BTreeMap<String, AppearanceLibrary>,
    #[serde(default)]
    pub claims: Vec<CapabilityClaim>,
}

impl Default for AppearanceInventory {
    fn default() -> Self {
        Self {
            schema: default_schema(),
            libraries: BTreeMap::new(),
            claims: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LibrarySelectionStatus {
    pub slot: String,
    pub library: String,
    pub item: String,
    pub source: String,
    pub revision: Option<String>,
    pub installation_cadence: Cadence,
    pub selection_cadence: Cadence,
    pub installed: bool,
    pub effective: bool,
    pub pending_session: bool,
}

impl AppearanceInventory {
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let Some(path) = path else {
            return Ok(Self::default());
        };
        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read appearance inventory {}", path.display()))?;
        let inventory: Self = toml::from_str(&raw)
            .with_context(|| format!("failed to parse appearance inventory {}", path.display()))?;
        if inventory.schema != 1 {
            bail!(
                "unsupported appearance inventory schema {}; expected 1",
                inventory.schema
            );
        }
        Ok(inventory)
    }

    pub fn validate_bundle(&self, bundle: &ThemeBundle) -> Result<Vec<LibrarySelectionStatus>> {
        let selections = [
            ("cursor", LibraryKind::Cursor, bundle.cursor.as_ref()),
            (
                "typography",
                LibraryKind::Typography,
                bundle.typography.as_ref(),
            ),
            ("icons", LibraryKind::Icon, bundle.icons.as_ref()),
            (
                "wallpaper",
                LibraryKind::Wallpaper,
                bundle.wallpaper.as_ref(),
            ),
        ];
        selections
            .into_iter()
            .filter_map(|(slot, expected, selection)| {
                selection.map(|selection| self.validate_selection(slot, expected, selection))
            })
            .collect()
    }

    fn validate_selection(
        &self,
        slot: &str,
        expected: LibraryKind,
        selection: &LibrarySelection,
    ) -> Result<LibrarySelectionStatus> {
        let library = self.libraries.get(&selection.library).with_context(|| {
            format!(
                "{slot} references unknown appearance library '{}'",
                selection.library
            )
        })?;
        if library.kind != expected {
            bail!(
                "{slot} requires a {:?} library, but '{}' is {:?}",
                expected,
                selection.library,
                library.kind
            );
        }
        let item = library.items.get(&selection.item).with_context(|| {
            format!(
                "{slot} references unknown item '{}' in library '{}'",
                selection.item, selection.library
            )
        })?;
        if !item.installed {
            bail!(
                "{slot} item '{}/{}' is not installed; a Nix rebuild is required",
                selection.library,
                selection.item
            );
        }
        if !item.effective {
            match item.selection_cadence {
                Cadence::Rebuild => bail!(
                    "{slot} item '{}/{}' is not effective; activate its Nix configuration first",
                    selection.library,
                    selection.item
                ),
                Cadence::Runtime => bail!(
                    "{slot} item '{}/{}' is not effective; its runtime selection failed",
                    selection.library,
                    selection.item
                ),
                Cadence::Session => {}
            }
        }
        for consumer in &item.required_consumers {
            if item
                .consumers
                .get(consumer)
                .is_none_or(|owner| owner.is_empty())
            {
                bail!(
                    "{slot} item '{}/{}' is missing selection coverage for consumer '{consumer}'",
                    selection.library,
                    selection.item
                );
            }
        }
        Ok(LibrarySelectionStatus {
            slot: slot.into(),
            library: selection.library.clone(),
            item: selection.item.clone(),
            source: library.source.clone(),
            revision: library.revision.clone(),
            installation_cadence: library.installation_cadence,
            selection_cadence: item.selection_cadence,
            installed: item.installed,
            effective: item.effective,
            pending_session: item.selection_cadence == Cadence::Session && !item.effective,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cursor_bundle(library: &str, item: &str) -> ThemeBundle {
        ThemeBundle {
            structure: "glass".into(),
            palette: "fallback".into(),
            cursor: Some(LibrarySelection {
                library: library.into(),
                item: item.into(),
            }),
            typography: None,
            icons: None,
            wallpaper: None,
        }
    }

    #[test]
    fn unknown_library_item_and_wrong_kind_are_rejected() {
        let bundle = cursor_bundle("stylix-cursors", "bibata");
        let error = AppearanceInventory::default()
            .validate_bundle(&bundle)
            .unwrap_err();
        assert!(error.to_string().contains("unknown appearance library"));

        let mut inventory = AppearanceInventory::default();
        inventory.libraries.insert(
            "stylix-cursors".into(),
            AppearanceLibrary {
                kind: LibraryKind::Cursor,
                source: "stylix.cursor".into(),
                revision: None,
                installation_cadence: Cadence::Rebuild,
                items: BTreeMap::new(),
            },
        );
        let error = inventory.validate_bundle(&bundle).unwrap_err();
        assert!(error.to_string().contains("unknown item"));

        inventory.libraries.get_mut("stylix-cursors").unwrap().kind = LibraryKind::Typography;
        let error = inventory.validate_bundle(&bundle).unwrap_err();
        assert!(error.to_string().contains("requires a Cursor library"));
    }

    #[test]
    fn qualified_cursor_requires_every_declared_consumer() {
        let mut inventory = AppearanceInventory::default();
        inventory.libraries.insert(
            "stylix-cursors".into(),
            AppearanceLibrary {
                kind: LibraryKind::Cursor,
                source: "stylix.cursor".into(),
                revision: None,
                installation_cadence: Cadence::Rebuild,
                items: BTreeMap::from([(
                    "bibata".into(),
                    LibraryItem {
                        installed: true,
                        effective: true,
                        selection_cadence: Cadence::Session,
                        required_consumers: vec!["gtk".into(), "niri".into()],
                        consumers: BTreeMap::from([("gtk".into(), "stylix".into())]),
                        values: BTreeMap::new(),
                    },
                )]),
            },
        );
        let bundle = ThemeBundle {
            structure: "glass".into(),
            palette: "fallback".into(),
            cursor: Some(LibrarySelection {
                library: "stylix-cursors".into(),
                item: "bibata".into(),
            }),
            typography: None,
            icons: None,
            wallpaper: None,
        };
        let error = inventory.validate_bundle(&bundle).unwrap_err();
        assert!(error.to_string().contains("consumer 'niri'"));
    }

    #[test]
    fn ineffective_session_selection_is_pending() {
        let mut inventory = AppearanceInventory::default();
        inventory.libraries.insert(
            "stylix-cursors".into(),
            AppearanceLibrary {
                kind: LibraryKind::Cursor,
                source: "stylix.cursor".into(),
                revision: None,
                installation_cadence: Cadence::Rebuild,
                items: BTreeMap::from([(
                    "bibata".into(),
                    LibraryItem {
                        installed: true,
                        effective: false,
                        selection_cadence: Cadence::Session,
                        required_consumers: Vec::new(),
                        consumers: BTreeMap::new(),
                        values: BTreeMap::new(),
                    },
                )]),
            },
        );
        let bundle = ThemeBundle {
            structure: "glass".into(),
            palette: "fallback".into(),
            cursor: Some(LibrarySelection {
                library: "stylix-cursors".into(),
                item: "bibata".into(),
            }),
            typography: None,
            icons: None,
            wallpaper: None,
        };

        let status = inventory.validate_bundle(&bundle).unwrap();
        assert!(status[0].pending_session);
        assert!(!status[0].effective);
    }

    #[test]
    fn ineffective_rebuild_selection_requires_activation() {
        let mut inventory = AppearanceInventory::default();
        inventory.libraries.insert(
            "stylix-cursors".into(),
            AppearanceLibrary {
                kind: LibraryKind::Cursor,
                source: "stylix.cursor".into(),
                revision: None,
                installation_cadence: Cadence::Rebuild,
                items: BTreeMap::from([(
                    "bibata".into(),
                    LibraryItem {
                        installed: true,
                        effective: false,
                        selection_cadence: Cadence::Rebuild,
                        required_consumers: Vec::new(),
                        consumers: BTreeMap::new(),
                        values: BTreeMap::new(),
                    },
                )]),
            },
        );
        let bundle = ThemeBundle {
            structure: "glass".into(),
            palette: "fallback".into(),
            cursor: Some(LibrarySelection {
                library: "stylix-cursors".into(),
                item: "bibata".into(),
            }),
            typography: None,
            icons: None,
            wallpaper: None,
        };

        let error = inventory.validate_bundle(&bundle).unwrap_err();
        assert!(error.to_string().contains("activate its Nix configuration"));
    }
}
