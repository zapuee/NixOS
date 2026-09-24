use crate::Palette;
use anyhow::Result;
use std::path::PathBuf;

pub trait PaletteProvider: Send + Sync {
    fn id(&self) -> &str;
    fn load(&self) -> Result<Palette>;
    fn watch_paths(&self) -> Vec<PathBuf> {
        Vec::new()
    }
}
