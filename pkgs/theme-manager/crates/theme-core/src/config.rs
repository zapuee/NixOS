use crate::Capability;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LibrarySelection {
    pub library: String,
    pub item: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ThemeBundle {
    pub structure: String,
    pub palette: String,
    #[serde(default)]
    pub cursor: Option<LibrarySelection>,
    #[serde(default)]
    pub typography: Option<LibrarySelection>,
    #[serde(default)]
    pub icons: Option<LibrarySelection>,
    #[serde(default)]
    pub wallpaper: Option<LibrarySelection>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicationContract {
    #[serde(default)]
    pub required: BTreeSet<Capability>,
    #[serde(default)]
    pub optional: BTreeSet<Capability>,
}
