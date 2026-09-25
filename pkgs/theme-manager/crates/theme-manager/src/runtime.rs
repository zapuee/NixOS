use crate::{
    adapters,
    config::{
        ApplicationWire, ColorPaletteConfig, ColorProviderConfig, Config, NoctaliaExecution,
        PaletteSource, validate_relative_output,
    },
    providers,
};
use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};
use theme_core::{
    AppearanceInventory, CapabilityPlan, LibrarySelectionStatus, Profile, ResolvedTheme,
    build_plan,
    fs::{contents_changed, write_if_changed},
    paths::{canonical_output_path, expand_path},
    resolve, state,
};

static NEXT_STAGING: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Serialize)]
pub struct ThemeSummary {
    pub name: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct StructureSummary {
    pub name: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaletteSummary {
    pub name: String,
    pub provider: String,
    pub source: PaletteSource,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct WireApplyReport {
    pub wire: String,
    pub changed: Vec<PathBuf>,
    pub unchanged: Vec<PathBuf>,
    pub delegated: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApplyReport {
    pub theme: String,
    pub structure: String,
    pub palette: String,
    pub dry_run: bool,
    pub capability_plan: CapabilityPlan,
    pub library_selections: Vec<LibrarySelectionStatus>,
    pub wires: Vec<WireApplyReport>,
}

struct Prepared {
    report: ApplyReport,
    artifacts: Vec<PreparedArtifact>,
    delegated_wires: Vec<String>,
}

struct PreparedArtifact {
    wire: String,
    path: PathBuf,
    contents: String,
}

pub struct Runtime {
    pub config: Config,
    pub config_path: PathBuf,
    pub inventory: AppearanceInventory,
}

impl Runtime {
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let (config, config_path) = Config::load(path)?;
        let inventory_path = config.appearance_inventory_file()?;
        let inventory = AppearanceInventory::load(inventory_path.as_deref())?;
        Ok(Self {
            config,
            config_path,
            inventory,
        })
    }

    pub fn active_theme(&self) -> Result<Option<String>> {
        let current = state::read(&self.config.state_file()?)?;
        Ok(current
            .active_theme
            .or_else(|| self.config.default_theme.clone()))
    }

    pub fn themes(&self) -> Result<Vec<ThemeSummary>> {
        let active = self.active_theme()?;
        Ok(self
            .config
            .theme_bundles
            .keys()
            .map(|name| ThemeSummary {
                name: name.clone(),
                active: active.as_deref() == Some(name),
            })
            .collect())
    }

    pub fn structures(&self) -> Result<Vec<StructureSummary>> {
        let active = self
            .active_bundle()?
            .map(|(_, bundle)| bundle.structure.as_str());
        let mut names = Vec::new();
        let dir = self.config.profiles_dir()?;
        if dir.is_dir() {
            for entry in fs::read_dir(&dir)
                .with_context(|| format!("failed to list structures in {}", dir.display()))?
            {
                let path = entry?.path();
                if path.extension().and_then(|value| value.to_str()) == Some("toml")
                    && let Some(name) = path.file_stem().and_then(|value| value.to_str())
                {
                    names.push(StructureSummary {
                        name: name.into(),
                        active: active == Some(name),
                    });
                }
            }
        }
        names.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(names)
    }

    pub fn palettes(&self) -> Result<Vec<PaletteSummary>> {
        let active = self
            .active_bundle()?
            .map(|(_, bundle)| bundle.palette.as_str());
        Ok(self
            .config
            .color_palettes
            .iter()
            .map(|(name, palette)| PaletteSummary {
                name: name.clone(),
                provider: palette.provider.clone(),
                source: palette.source,
                active: active == Some(name.as_str()),
            })
            .collect())
    }

    pub fn plan(&self, theme: &str) -> Result<(CapabilityPlan, Vec<LibrarySelectionStatus>)> {
        let bundle = self
            .config
            .theme_bundles
            .get(theme)
            .with_context(|| format!("unknown theme bundle '{theme}'"))?;
        let selections = self.inventory.validate_bundle(bundle)?;
        let mut claims = self.inventory.claims.clone();
        for (name, wire) in &self.config.application_wires {
            if wire.enabled() {
                self.validate_wire_palette(name, wire)?;
                claims.extend(adapters::claims(name, wire)?);
            }
        }
        Ok((
            build_plan(&self.config.application_contracts, claims),
            selections,
        ))
    }

    pub fn check(&self, theme: &str) -> Result<ApplyReport> {
        let prepared = self.prepare(theme, true)?;
        prepared.report.capability_plan.require_complete()?;
        Ok(prepared.report)
    }

    pub fn apply(&self, theme: &str, dry_run: bool) -> Result<ApplyReport> {
        let state_path = self.config.state_file()?;
        state::with_lock(&state_path, || {
            let mut prepared = self.prepare(theme, dry_run)?;
            prepared.report.capability_plan.require_complete()?;
            if dry_run {
                return Ok(prepared.report);
            }

            let desired_paths = prepared
                .artifacts
                .iter()
                .map(|artifact| artifact.path.clone())
                .collect::<BTreeSet<_>>();
            let mut reports = BTreeMap::<String, WireApplyReport>::new();
            for artifact in prepared.artifacts {
                let changed = contents_changed(&artifact.path, &artifact.contents)?;
                let report =
                    reports
                        .entry(artifact.wire.clone())
                        .or_insert_with(|| WireApplyReport {
                            wire: artifact.wire,
                            changed: Vec::new(),
                            unchanged: Vec::new(),
                            delegated: false,
                        });
                if changed {
                    write_if_changed(&artifact.path, &artifact.contents)?;
                    report.changed.push(artifact.path);
                } else {
                    report.unchanged.push(artifact.path);
                }
            }
            self.remove_stale_outputs(&desired_paths, &mut reports)?;

            let mut current = state::read(&state_path)?;
            current.active_theme = Some(theme.into());
            state::write(&state_path, &current)?;

            if !prepared.delegated_wires.is_empty() {
                apply_shell_templates(&prepared.delegated_wires)?;
                for wire in prepared.delegated_wires {
                    reports
                        .entry(wire.clone())
                        .or_insert_with(|| WireApplyReport {
                            wire,
                            changed: Vec::new(),
                            unchanged: Vec::new(),
                            delegated: true,
                        })
                        .delegated = true;
                }
            }
            prepared.report.wires = reports.into_values().collect();
            Ok(prepared.report)
        })
    }

    pub fn watch_paths(&self) -> Result<Vec<PathBuf>> {
        let mut paths = vec![
            self.config_path.clone(),
            self.config.profiles_dir()?,
            self.config.state_file()?,
        ];
        if let Some(inventory) = self.config.appearance_inventory_file()? {
            paths.push(inventory);
        }
        let palettes = self
            .active_bundle()?
            .map(|(_, bundle)| self.palette_names(bundle))
            .unwrap_or_default();
        for name in palettes {
            let palette = &self.config.color_palettes[&name];
            let path = match palette.source {
                PaletteSource::JsonFile | PaletteSource::ShellJson => palette.path.as_deref(),
                PaletteSource::Image => palette.image.as_deref(),
                PaletteSource::ThemeJson => palette.theme_json.as_deref(),
                PaletteSource::Static => None,
            };
            if let Some(path) = path {
                paths.push(expand_path(path)?);
            }
        }
        paths.sort();
        paths.dedup();
        Ok(paths)
    }

    fn prepare(&self, theme: &str, dry_run: bool) -> Result<Prepared> {
        let bundle = self
            .config
            .theme_bundles
            .get(theme)
            .with_context(|| format!("unknown theme bundle '{theme}'"))?;
        let (plan, selections) = self.plan(theme)?;
        plan.require_complete()?;

        let mut palettes = BTreeMap::new();
        for name in self.palette_names(bundle) {
            palettes.insert(
                name.clone(),
                providers::load(&self.config, &name)
                    .with_context(|| format!("failed to load palette '{name}'"))?,
            );
        }
        let profile = self.load_profile(&bundle.structure)?;
        let mut themes = BTreeMap::<String, ResolvedTheme>::new();
        for (name, palette) in &palettes {
            themes.insert(
                name.clone(),
                resolve::resolve(&profile, &palette.colors)
                    .with_context(|| format!("failed to resolve palette '{name}'"))?,
            );
        }

        let generated_dir = self.config.generated_dir()?;
        let mut artifacts = Vec::new();
        let mut delegated_wires = Vec::new();
        let mut owners = BTreeMap::<PathBuf, String>::new();
        for (name, wire) in &self.config.application_wires {
            if !wire.enabled() {
                continue;
            }
            let palette_name = wire.palette_override().unwrap_or(&bundle.palette);
            match wire {
                ApplicationWire::NoctaliaTemplate {
                    execution: NoctaliaExecution::Shell,
                    ..
                } => delegated_wires.push(name.clone()),
                ApplicationWire::NoctaliaTemplate {
                    execution: NoctaliaExecution::Headless,
                    template,
                    output,
                    ..
                } => {
                    let palette = &self.config.color_palettes[palette_name];
                    let contents = render_headless_template(
                        name,
                        palette_name,
                        palette,
                        adapters::headless_template(template)?,
                        output.as_deref().expect("validated output"),
                    )?;
                    let path = safe_output_path(&generated_dir, output.as_deref().unwrap())?;
                    register_output(&mut owners, name, &path)?;
                    artifacts.push(PreparedArtifact {
                        wire: name.clone(),
                        path,
                        contents,
                    });
                }
                _ => {
                    let theme = &themes[palette_name];
                    if let Some(rendered) = adapters::render(name, wire, theme)? {
                        let path = safe_output_path(&generated_dir, &rendered.relative_path)?;
                        register_output(&mut owners, name, &path)?;
                        artifacts.push(PreparedArtifact {
                            wire: rendered.wire,
                            path,
                            contents: rendered.contents,
                        });
                    }
                }
            }
        }

        let mut reports = BTreeMap::<String, WireApplyReport>::new();
        for artifact in &artifacts {
            let report = reports
                .entry(artifact.wire.clone())
                .or_insert_with(|| WireApplyReport {
                    wire: artifact.wire.clone(),
                    changed: Vec::new(),
                    unchanged: Vec::new(),
                    delegated: false,
                });
            if contents_changed(&artifact.path, &artifact.contents)? {
                report.changed.push(artifact.path.clone());
            } else {
                report.unchanged.push(artifact.path.clone());
            }
        }
        for wire in &delegated_wires {
            reports.insert(
                wire.clone(),
                WireApplyReport {
                    wire: wire.clone(),
                    changed: Vec::new(),
                    unchanged: Vec::new(),
                    delegated: !dry_run,
                },
            );
        }
        Ok(Prepared {
            report: ApplyReport {
                theme: theme.into(),
                structure: bundle.structure.clone(),
                palette: bundle.palette.clone(),
                dry_run,
                capability_plan: plan,
                library_selections: selections,
                wires: reports.into_values().collect(),
            },
            artifacts,
            delegated_wires,
        })
    }

    fn active_bundle(&self) -> Result<Option<(String, &theme_core::ThemeBundle)>> {
        let Some(active) = self.active_theme()? else {
            return Ok(None);
        };
        let bundle = self
            .config
            .theme_bundles
            .get(&active)
            .with_context(|| format!("active/default theme '{active}' is not declared"))?;
        Ok(Some((active, bundle)))
    }

    fn palette_names(&self, bundle: &theme_core::ThemeBundle) -> BTreeSet<String> {
        let mut names = BTreeSet::from([bundle.palette.clone()]);
        for wire in self
            .config
            .application_wires
            .values()
            .filter(|wire| wire.enabled())
        {
            if let Some(palette) = wire.palette_override() {
                names.insert(palette.into());
            }
        }
        names
    }

    fn load_profile(&self, name: &str) -> Result<Profile> {
        validate_simple_name("structure", name)?;
        let path = self.config.profiles_dir()?.join(format!("{name}.toml"));
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("failed to read structure profile {}", path.display()))?;
        let profile: Profile = toml::from_str(&raw)
            .with_context(|| format!("failed to parse structure profile {}", path.display()))?;
        if profile.name != name {
            bail!(
                "structure file '{name}.toml' declares name = '{}'",
                profile.name
            );
        }
        Ok(profile)
    }

    fn validate_wire_palette(&self, name: &str, wire: &ApplicationWire) -> Result<()> {
        let ApplicationWire::NoctaliaTemplate {
            palette, execution, ..
        } = wire
        else {
            return Ok(());
        };
        let palette_config = &self.config.color_palettes[palette];
        let provider = &self.config.color_providers[&palette_config.provider];
        if !matches!(provider, ColorProviderConfig::Noctalia) {
            bail!(
                "Noctalia template wire '{name}' requires a Noctalia-backed palette; '{palette}' uses '{}'",
                palette_config.provider
            );
        }
        match execution {
            NoctaliaExecution::Shell if palette_config.source != PaletteSource::ShellJson => {
                bail!("shell-driven Noctalia wire '{name}' requires a shell-json palette authority")
            }
            NoctaliaExecution::Headless
                if !matches!(
                    palette_config.source,
                    PaletteSource::Image | PaletteSource::ThemeJson
                ) =>
            {
                bail!("headless Noctalia wire '{name}' requires an image or theme-json palette")
            }
            _ => {}
        }
        Ok(())
    }

    fn remove_stale_outputs(
        &self,
        desired_paths: &BTreeSet<PathBuf>,
        reports: &mut BTreeMap<String, WireApplyReport>,
    ) -> Result<()> {
        let generated = self.config.generated_dir()?;
        if !generated.is_dir() {
            return Ok(());
        }
        let mut removed = Vec::new();
        remove_stale_entries(&generated, desired_paths, &mut removed)?;
        if !removed.is_empty() {
            reports.insert(
                "cleanup".into(),
                WireApplyReport {
                    wire: "cleanup".into(),
                    changed: removed,
                    unchanged: Vec::new(),
                    delegated: false,
                },
            );
        }
        Ok(())
    }
}

fn remove_stale_entries(
    directory: &Path,
    desired_paths: &BTreeSet<PathBuf>,
    removed: &mut Vec<PathBuf>,
) -> Result<()> {
    for entry in fs::read_dir(directory)
        .with_context(|| format!("failed to list generated directory {}", directory.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let kind = entry.file_type()?;
        if kind.is_dir() && !kind.is_symlink() {
            remove_stale_entries(&path, desired_paths, removed)?;
            if fs::read_dir(&path)?.next().is_none() {
                fs::remove_dir(&path).with_context(|| {
                    format!("failed to remove empty directory {}", path.display())
                })?;
            }
        } else if !desired_paths.contains(&path) {
            fs::remove_file(&path)
                .with_context(|| format!("failed to remove stale output {}", path.display()))?;
            removed.push(path);
        }
    }
    Ok(())
}

fn validate_simple_name(kind: &str, name: &str) -> Result<()> {
    if name.is_empty()
        || name == "."
        || name == ".."
        || name.contains(['/', '\\'])
        || name.chars().any(char::is_control)
    {
        bail!("invalid {kind} name '{name}'");
    }
    Ok(())
}

fn safe_output_path(root: &Path, relative: &str) -> Result<PathBuf> {
    validate_relative_output("generated output", relative)?;
    let root_key = canonical_output_path(root)?;
    let output = root.join(relative);
    let output_key = canonical_output_path(&output)?;
    if !output_key.starts_with(&root_key) {
        bail!(
            "generated output {} escapes generated directory {}",
            output.display(),
            root.display()
        );
    }
    Ok(output)
}

fn register_output(owners: &mut BTreeMap<PathBuf, String>, wire: &str, path: &Path) -> Result<()> {
    let key = canonical_output_path(path)?;
    if let Some(previous) = owners.insert(key, wire.into()) {
        bail!(
            "application wires '{previous}' and '{wire}' both write {}",
            path.display()
        );
    }
    Ok(())
}

fn render_headless_template(
    wire: &str,
    palette_name: &str,
    palette: &ColorPaletteConfig,
    template_contents: &str,
    output: &str,
) -> Result<String> {
    let staging = create_staging_dir()?;
    let template = staging.join("template");
    fs::write(&template, template_contents)
        .with_context(|| format!("failed to stage headless Noctalia wire '{wire}' template"))?;
    let staged_output = staging.join(output);
    if let Some(parent) = staged_output.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut args = providers::noctalia_source_args(palette_name, palette)?;
    args.extend([
        "--render".into(),
        format!("{}:{}", template.display(), staged_output.display()),
        "--default-mode".into(),
        palette.mode.key().into(),
    ]);
    let result = Command::new(providers::noctalia_command())
        .args(args)
        .output()
        .with_context(|| format!("headless Noctalia wire '{wire}' could not start the CLI"));
    let result = match result {
        Ok(result) => result,
        Err(error) => {
            let _ = fs::remove_dir_all(&staging);
            return Err(error);
        }
    };
    if !result.status.success() {
        let _ = fs::remove_dir_all(&staging);
        bail!(
            "headless Noctalia wire '{wire}' failed: {}",
            String::from_utf8_lossy(&result.stderr).trim()
        );
    }
    let contents = fs::read_to_string(&staged_output).with_context(|| {
        format!(
            "headless Noctalia wire '{wire}' did not render {}",
            staged_output.display()
        )
    });
    let _ = fs::remove_dir_all(staging);
    contents
}

fn create_staging_dir() -> Result<PathBuf> {
    for _ in 0..100 {
        let sequence = NEXT_STAGING.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "theme-manager-stage-{}-{sequence}",
            std::process::id()
        ));
        match fs::create_dir(&path) {
            Ok(()) => {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    fs::set_permissions(&path, fs::Permissions::from_mode(0o700))?;
                }
                return Ok(path);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error).context("failed to create staging directory"),
        }
    }
    bail!("failed to create a unique staging directory")
}

fn apply_shell_templates(wires: &[String]) -> Result<()> {
    let output = Command::new(providers::noctalia_command())
        .args(["msg", "templates-apply"])
        .output()
        .context("failed to request shell-driven Noctalia templates")?;
    if !output.status.success() {
        bail!(
            "delegated Noctalia templates ({}) failed: {}",
            wires.join(", "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

pub fn parent_or_current(path: &Path) -> PathBuf {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::{register_output, safe_output_path};
    use std::{collections::BTreeMap, fs, path::PathBuf};

    #[test]
    fn two_wires_cannot_own_one_output() {
        let mut owners = BTreeMap::new();
        let output = PathBuf::from("generated/application.conf");
        register_output(&mut owners, "first", &output).unwrap();
        let error = register_output(&mut owners, "second", &output).unwrap_err();
        assert!(error.to_string().contains("both write"));
    }

    #[cfg(unix)]
    #[test]
    fn generated_outputs_cannot_escape_through_symlinks() {
        use std::os::unix::fs::symlink;
        let root = std::env::temp_dir().join(format!(
            "theme-manager-containment-test-{}",
            std::process::id()
        ));
        let generated = root.join("generated");
        let outside = root.join("outside");
        fs::create_dir_all(&generated).unwrap();
        fs::create_dir_all(&outside).unwrap();
        symlink(&outside, generated.join("escape")).unwrap();
        assert!(safe_output_path(&generated, "escape/theme.css").is_err());
        fs::remove_dir_all(root).unwrap();
    }
}
