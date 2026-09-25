use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
use theme_core::{ApplicationContract, ThemeBundle, paths::expand_path};

fn default_profiles_dir() -> String {
    "$XDG_CONFIG_HOME/theme-manager/profiles".into()
}

fn default_state_file() -> String {
    "$XDG_STATE_HOME/theme-manager/state.toml".into()
}

fn default_generated_dir() -> String {
    "$XDG_CONFIG_HOME/theme-manager/generated".into()
}

fn default_true() -> bool {
    true
}

fn default_global_role() -> String {
    "compositor".into()
}

fn default_terminal_role() -> String {
    "terminal".into()
}

fn default_mode() -> ColorMode {
    ColorMode::Dark
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default = "default_profiles_dir")]
    pub profiles_dir: String,
    #[serde(default = "default_state_file")]
    pub state_file: String,
    #[serde(default = "default_generated_dir")]
    pub generated_dir: String,
    #[serde(default)]
    pub appearance_inventory_file: Option<String>,
    #[serde(default)]
    pub default_theme: Option<String>,
    #[serde(default)]
    pub color_providers: BTreeMap<String, ColorProviderConfig>,
    #[serde(default)]
    pub color_palettes: BTreeMap<String, ColorPaletteConfig>,
    #[serde(default)]
    pub theme_bundles: BTreeMap<String, ThemeBundle>,
    #[serde(default)]
    pub application_contracts: BTreeMap<String, ApplicationContract>,
    #[serde(default)]
    pub application_wires: BTreeMap<String, ApplicationWire>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum ColorProviderConfig {
    Static,
    JsonFile,
    Noctalia,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PaletteSource {
    Static,
    JsonFile,
    ShellJson,
    Image,
    ThemeJson,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ColorMode {
    Dark,
    Light,
}

impl ColorMode {
    pub fn flag(self) -> &'static str {
        match self {
            Self::Dark => "--dark",
            Self::Light => "--light",
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ColorPaletteConfig {
    pub provider: String,
    pub source: PaletteSource,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub theme_json: Option<String>,
    #[serde(default)]
    pub scheme: Option<String>,
    #[serde(default = "default_mode")]
    pub mode: ColorMode,
    #[serde(default)]
    pub pure_black: bool,
    #[serde(default)]
    pub colors: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OpacityOwner {
    Application,
    Compositor,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NiriWindow {
    pub application: String,
    pub role: String,
    pub app_id: String,
    pub opacity_owner: OpacityOwner,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NoctaliaExecution {
    Headless,
    Shell,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "adapter", rename_all = "kebab-case", deny_unknown_fields)]
pub enum ApplicationWire {
    Niri {
        #[serde(default = "default_true")]
        enabled: bool,
        #[serde(default)]
        palette: Option<String>,
        #[serde(default = "default_global_role")]
        global_role: String,
        #[serde(default)]
        windows: Vec<NiriWindow>,
    },
    Foot {
        #[serde(default = "default_true")]
        enabled: bool,
        #[serde(default)]
        palette: Option<String>,
        #[serde(default = "default_terminal_role")]
        role: String,
    },
    NoctaliaTemplate {
        #[serde(default = "default_true")]
        enabled: bool,
        palette: String,
        execution: NoctaliaExecution,
        template: String,
        #[serde(default)]
        output: Option<String>,
    },
}

impl ApplicationWire {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Niri { enabled, .. }
            | Self::Foot { enabled, .. }
            | Self::NoctaliaTemplate { enabled, .. } => *enabled,
        }
    }

    pub fn palette_override(&self) -> Option<&str> {
        match self {
            Self::Niri { palette, .. } | Self::Foot { palette, .. } => palette.as_deref(),
            Self::NoctaliaTemplate { palette, .. } => Some(palette),
        }
    }
}

impl Config {
    pub fn load(path: Option<&Path>) -> Result<(Self, PathBuf)> {
        let path = match path {
            Some(path) => path.to_path_buf(),
            None => theme_core::paths::default_config_path()?,
        };
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("failed to read config {}", path.display()))?;
        let config: Self = toml::from_str(&raw)
            .with_context(|| format!("failed to parse config {}", path.display()))?;
        config.validate()?;
        Ok((config, path))
    }

    pub fn validate(&self) -> Result<()> {
        if self.color_providers.is_empty() {
            bail!("at least one color provider is required");
        }
        if self.color_palettes.is_empty() {
            bail!("at least one color palette is required");
        }
        if self.theme_bundles.is_empty() {
            bail!("at least one theme bundle is required");
        }
        if let Some(default) = &self.default_theme
            && !self.theme_bundles.contains_key(default)
        {
            bail!("default theme '{default}' is not declared");
        }
        for (name, palette) in &self.color_palettes {
            let provider = self
                .color_providers
                .get(&palette.provider)
                .with_context(|| {
                    format!(
                        "palette '{name}' references unknown provider '{}'",
                        palette.provider
                    )
                })?;
            validate_palette(name, palette, provider)?;
        }
        for (name, bundle) in &self.theme_bundles {
            if !self.color_palettes.contains_key(&bundle.palette) {
                bail!(
                    "theme bundle '{name}' references unknown palette '{}'",
                    bundle.palette
                );
            }
            validate_name("structure", &bundle.structure)?;
        }
        for (application, contract) in &self.application_contracts {
            if let Some(capability) = contract.required.intersection(&contract.optional).next() {
                bail!(
                    "application contract '{application}' lists '{capability}' as both required and optional"
                );
            }
        }
        for (name, wire) in &self.application_wires {
            validate_wire(name, wire, self)?;
        }
        Ok(())
    }

    pub fn profiles_dir(&self) -> Result<PathBuf> {
        expand_path(&self.profiles_dir)
    }

    pub fn state_file(&self) -> Result<PathBuf> {
        expand_path(&self.state_file)
    }

    pub fn generated_dir(&self) -> Result<PathBuf> {
        expand_path(&self.generated_dir)
    }

    pub fn appearance_inventory_file(&self) -> Result<Option<PathBuf>> {
        self.appearance_inventory_file
            .as_deref()
            .map(expand_path)
            .transpose()
    }
}

fn validate_palette(
    name: &str,
    palette: &ColorPaletteConfig,
    provider: &ColorProviderConfig,
) -> Result<()> {
    let valid = matches!(
        (provider, palette.source),
        (ColorProviderConfig::Static, PaletteSource::Static)
            | (ColorProviderConfig::JsonFile, PaletteSource::JsonFile)
            | (ColorProviderConfig::Noctalia, PaletteSource::ShellJson)
            | (ColorProviderConfig::Noctalia, PaletteSource::Image)
            | (ColorProviderConfig::Noctalia, PaletteSource::ThemeJson)
    );
    if !valid {
        bail!(
            "palette '{name}' source '{:?}' is incompatible with provider '{}'",
            palette.source,
            palette.provider
        );
    }
    match palette.source {
        PaletteSource::Static if palette.colors.is_empty() => {
            bail!("static palette '{name}' must declare colors")
        }
        PaletteSource::JsonFile | PaletteSource::ShellJson if palette.path.is_none() => {
            bail!("palette '{name}' must declare path")
        }
        PaletteSource::Image if palette.image.is_none() => {
            bail!("palette '{name}' must declare image")
        }
        PaletteSource::ThemeJson if palette.theme_json.is_none() => {
            bail!("palette '{name}' must declare theme_json")
        }
        _ => {}
    }
    if !matches!(palette.source, PaletteSource::Static) && !palette.colors.is_empty() {
        bail!("palette '{name}' may declare colors only with source = 'static'");
    }
    if let Some(scheme) = &palette.scheme {
        if !matches!(palette.source, PaletteSource::Image) {
            bail!("palette '{name}' may declare scheme only with source = 'image'");
        }
        validate_noctalia_scheme(name, scheme)?;
    }
    Ok(())
}

pub fn validate_noctalia_scheme(palette: &str, scheme: &str) -> Result<()> {
    const SCHEMES: &[&str] = &[
        "m3-tonal-spot",
        "m3-content",
        "m3-fruit-salad",
        "m3-rainbow",
        "m3-monochrome",
        "vibrant",
        "faithful",
        "soft",
        "dysfunctional",
        "muted",
    ];
    if !SCHEMES.contains(&scheme) {
        bail!("palette '{palette}' has unknown Noctalia scheme '{scheme}'");
    }
    Ok(())
}

fn validate_wire(name: &str, wire: &ApplicationWire, config: &Config) -> Result<()> {
    if let Some(palette) = wire.palette_override()
        && !config.color_palettes.contains_key(palette)
    {
        bail!("application wire '{name}' references unknown palette '{palette}'");
    }
    if let ApplicationWire::NoctaliaTemplate {
        execution,
        template,
        output,
        ..
    } = wire
    {
        match execution {
            NoctaliaExecution::Shell => {
                if output.is_some() {
                    bail!("shell-driven Noctalia wire '{name}' may not declare output");
                }
                if !matches!(
                    template.as_str(),
                    "user:foot-colors" | "builtin:starship" | "user:pywalfox"
                ) {
                    bail!("Noctalia wire '{name}' uses unaudited shell template '{template}'");
                }
            }
            NoctaliaExecution::Headless => {
                if !matches!(
                    template.as_str(),
                    "repository:palette" | "repository:foot-colors" | "repository:pywalfox"
                ) {
                    bail!("Noctalia wire '{name}' uses unaudited headless template '{template}'");
                }
                if output.is_none() {
                    bail!("headless Noctalia wire '{name}' requires a relative output");
                }
                validate_relative_output(name, output.as_deref().unwrap())?;
            }
        }
    }
    Ok(())
}

fn validate_name(kind: &str, name: &str) -> Result<()> {
    if name.is_empty()
        || name == "."
        || name == ".."
        || name.contains('/')
        || name.contains('\\')
        || name.chars().any(char::is_control)
    {
        bail!("invalid {kind} name '{name}'");
    }
    Ok(())
}

pub fn validate_relative_output(wire: &str, output: &str) -> Result<()> {
    let path = Path::new(output);
    if path.is_absolute()
        || output.is_empty()
        || path.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir
                    | std::path::Component::RootDir
                    | std::path::Component::Prefix(_)
            )
        })
    {
        bail!("Noctalia wire '{wire}' output must stay beneath generated_dir");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Config, validate_noctalia_scheme};

    const BASE: &str = r##"
default_theme = "glass"

[color_providers.static]
kind = "static"

[color_palettes.fallback]
provider = "static"
source = "static"
[color_palettes.fallback.colors]
primary = "#ffffff"

[theme_bundles.glass]
structure = "glass"
palette = "fallback"
"##;

    #[test]
    fn typed_config_rejects_unknown_fields() {
        let error = toml::from_str::<Config>(&format!("{BASE}\nextra = true\n")).unwrap_err();
        assert!(error.to_string().contains("unknown field"));
    }

    #[test]
    fn bundle_assets_must_use_qualified_library_selections() {
        let source = BASE.replace(
            "palette = \"fallback\"",
            "palette = \"fallback\"\ncursor = \"bibata-modern-ice\"",
        );
        let error = toml::from_str::<Config>(&source).unwrap_err();
        assert!(error.to_string().contains("invalid type"));
    }

    #[test]
    fn headless_template_hooks_are_not_configuration() {
        let source = format!(
            "{BASE}\n[application_wires.example]\nadapter = \"noctalia-template\"\npalette = \"fallback\"\nexecution = \"headless\"\ntemplate = \"repository:palette\"\noutput = \"output\"\npost_action = \"run-me\"\n"
        );
        let error = toml::from_str::<Config>(&source).unwrap_err();
        assert!(error.to_string().contains("unknown field"));
    }

    #[test]
    fn noctalia_schemes_are_closed_and_named_in_errors() {
        assert!(validate_noctalia_scheme("wallpaper", "muted").is_ok());
        let error = validate_noctalia_scheme("wallpaper", "surprise").unwrap_err();
        assert!(error.to_string().contains("palette 'wallpaper'"));
    }
}
