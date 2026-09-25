use crate::config::{ColorPaletteConfig, ColorProviderConfig, Config, PaletteSource};
use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::{collections::BTreeMap, fs, process::Command};
use theme_core::{Palette, paths::expand_path};

#[derive(Debug, Clone)]
pub struct LoadedPalette {
    pub colors: Palette,
}

pub fn load(config: &Config, name: &str) -> Result<LoadedPalette> {
    let palette = config
        .color_palettes
        .get(name)
        .with_context(|| format!("unknown color palette '{name}'"))?;
    let provider = config
        .color_providers
        .get(&palette.provider)
        .expect("configuration validation checked provider references");
    let colors = match provider {
        ColorProviderConfig::Static => Palette {
            colors: palette.colors.clone(),
        },
        ColorProviderConfig::JsonFile => load_json_path(
            name,
            palette.path.as_deref().expect("validated JSON path"),
            palette.mode.key(),
        )?,
        ColorProviderConfig::Noctalia if palette.source == PaletteSource::ShellJson => {
            load_json_path(
                name,
                palette.path.as_deref().expect("validated shell JSON path"),
                palette.mode.key(),
            )?
        }
        ColorProviderConfig::Noctalia => load_noctalia(name, palette)?,
    };
    colors.validate().with_context(|| {
        format!(
            "provider '{}' returned an invalid palette '{name}'",
            palette.provider
        )
    })?;
    Ok(LoadedPalette { colors })
}

pub fn noctalia_source_args(name: &str, palette: &ColorPaletteConfig) -> Result<Vec<String>> {
    let mut args = vec!["theme".into()];
    match palette.source {
        PaletteSource::Image => {
            args.push(
                expand_path(palette.image.as_deref().expect("validated image"))?
                    .display()
                    .to_string(),
            );
            args.extend([
                "--scheme".into(),
                palette.scheme.as_deref().unwrap_or("m3-tonal-spot").into(),
            ]);
        }
        PaletteSource::ThemeJson => {
            args.extend([
                "--theme-json".into(),
                expand_path(palette.theme_json.as_deref().expect("validated theme JSON"))?
                    .display()
                    .to_string(),
            ]);
        }
        _ => bail!(
            "palette '{name}' cannot drive a headless Noctalia render; use source = 'image' or 'theme-json'"
        ),
    }
    args.push(palette.mode.flag().into());
    if palette.pure_black {
        args.push("--pure-black".into());
    }
    Ok(args)
}

pub fn noctalia_command() -> String {
    std::env::var("THEME_MANAGER_NOCTALIA").unwrap_or_else(|_| "noctalia".into())
}

fn load_noctalia(name: &str, palette: &ColorPaletteConfig) -> Result<Palette> {
    let output = Command::new(noctalia_command())
        .args(noctalia_source_args(name, palette)?)
        .output()
        .with_context(|| {
            format!(
                "Noctalia provider '{}' could not start the noctalia CLI for palette '{name}'",
                palette.provider
            )
        })?;
    if !output.status.success() {
        bail!(
            "Noctalia provider '{}' failed for palette '{name}': {}",
            palette.provider,
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    parse_json_palette(name, &output.stdout, palette.mode.key())
}

fn load_json_path(name: &str, path: &str, mode: &str) -> Result<Palette> {
    let path = expand_path(path)?;
    let raw = fs::read(&path)
        .with_context(|| format!("failed to read palette '{name}' from {}", path.display()))?;
    parse_json_palette(name, &raw, mode)
}

fn parse_json_palette(name: &str, raw: &[u8], mode: &str) -> Result<Palette> {
    let value: Value = serde_json::from_slice(raw)
        .with_context(|| format!("palette '{name}' is not valid JSON"))?;
    let mut object = value
        .as_object()
        .with_context(|| format!("palette '{name}' must be a JSON object"))?;
    if let Some(selected) = object.get(mode) {
        object = selected
            .as_object()
            .with_context(|| format!("palette '{name}' field '{mode}' must be a JSON object"))?;
    }
    let mut colors = object
        .iter()
        .filter_map(|(key, value)| value.as_str().map(|value| (key.clone(), value.into())))
        .collect::<BTreeMap<_, _>>();
    if colors.is_empty() {
        bail!("palette '{name}' contains no color strings");
    }
    if !colors.contains_key("background")
        && let Some(surface) = colors.get("surface").cloned()
    {
        colors.insert("background".into(), surface);
    }
    if !colors.contains_key("on_background")
        && let Some(on_surface) = colors.get("on_surface").cloned()
    {
        colors.insert("on_background".into(), on_surface);
    }
    Ok(Palette { colors })
}

#[cfg(test)]
mod tests {
    use super::{noctalia_source_args, parse_json_palette};
    use crate::config::{ColorMode, ColorPaletteConfig, PaletteSource};
    use std::collections::BTreeMap;

    #[test]
    fn normalizes_flat_and_dark_light_json() {
        let flat = parse_json_palette(
            "flat",
            br##"{"surface":"#000000","on_surface":"#ffffff"}"##,
            "dark",
        )
        .unwrap();
        assert_eq!(flat.get("background").unwrap(), "#000000");
        let modes = parse_json_palette(
            "modes",
            br##"{"dark":{"primary":"#111111"},"light":{"primary":"#eeeeee"}}"##,
            "light",
        )
        .unwrap();
        assert_eq!(modes.get("primary").unwrap(), "#eeeeee");
    }

    #[test]
    fn image_palettes_use_the_explicit_default_or_selected_scheme() {
        let mut palette = ColorPaletteConfig {
            provider: "noctalia".into(),
            source: PaletteSource::Image,
            path: None,
            image: Some("/tmp/wallpaper.png".into()),
            theme_json: None,
            scheme: None,
            mode: ColorMode::Dark,
            pure_black: false,
            colors: BTreeMap::new(),
        };
        let defaults = noctalia_source_args("wallpaper", &palette).unwrap();
        assert!(
            defaults
                .windows(2)
                .any(|pair| pair == ["--scheme", "m3-tonal-spot"])
        );

        palette.scheme = Some("muted".into());
        let selected = noctalia_source_args("wallpaper", &palette).unwrap();
        assert!(
            selected
                .windows(2)
                .any(|pair| pair == ["--scheme", "muted"])
        );
    }
}
