use crate::{
    Artifact, Config, GENERATED_MARKER, PaletteProvider, Profile, ResolvedTheme, Target,
    fs::contents_changed,
    paths::canonical_output_path,
    resolve, state,
    transaction::{self, Operation},
};
use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
};

#[derive(Debug, Clone, Serialize)]
pub struct ProfileSummary {
    pub name: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TargetApplyReport {
    pub target: String,
    pub changed: Vec<PathBuf>,
    pub unchanged: Vec<PathBuf>,
    pub forced: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApplyReport {
    pub profile: String,
    pub dry_run: bool,
    pub targets: Vec<TargetApplyReport>,
}

pub struct Engine {
    config: Config,
    provider: Box<dyn PaletteProvider>,
    targets: Vec<Box<dyn Target>>,
    disabled_targets: BTreeSet<String>,
}

impl Engine {
    pub fn new(
        config: Config,
        provider: Box<dyn PaletteProvider>,
        targets: Vec<Box<dyn Target>>,
    ) -> Self {
        Self {
            config,
            provider,
            targets,
            disabled_targets: BTreeSet::new(),
        }
    }

    pub fn with_disabled_targets(mut self, disabled_targets: BTreeSet<String>) -> Self {
        self.disabled_targets = disabled_targets;
        self
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
    pub fn provider_id(&self) -> &str {
        self.provider.id()
    }
    pub fn target_ids(&self) -> Vec<&str> {
        self.targets
            .iter()
            .filter(|target| !self.disabled_targets.contains(target.id()))
            .map(|target| target.id())
            .collect()
    }
    pub fn provider_watch_paths(&self) -> Vec<PathBuf> {
        self.provider.watch_paths()
    }

    pub fn active_profile(&self) -> Result<Option<String>> {
        let state = state::read(&self.config.state_file()?)?;
        Ok(state
            .active_profile
            .or_else(|| self.config.default_profile.clone()))
    }

    pub fn list_profiles(&self) -> Result<Vec<ProfileSummary>> {
        let dir = self.config.profiles_dir()?;
        let active = self.active_profile()?;
        let mut profiles = Vec::new();

        if !dir.exists() {
            return Ok(profiles);
        }

        for entry in fs::read_dir(&dir)
            .with_context(|| format!("failed to list profiles in {}", dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() || path.extension().and_then(|v| v.to_str()) != Some("toml") {
                continue;
            }
            let Some(name) = path.file_stem().and_then(|v| v.to_str()) else {
                continue;
            };
            profiles.push(ProfileSummary {
                name: name.to_string(),
                active: active.as_deref() == Some(name),
            });
        }

        profiles.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(profiles)
    }

    fn profile_path(&self, name: &str) -> Result<PathBuf> {
        if name.is_empty()
            || name == "."
            || name == ".."
            || name.contains('/')
            || name.contains('\\')
            || name.chars().any(char::is_control)
        {
            bail!("invalid profile name '{name}'");
        }
        Ok(self.config.profiles_dir()?.join(format!("{name}.toml")))
    }

    pub fn load_profile(&self, name: &str) -> Result<Profile> {
        let path = self.profile_path(name)?;
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("failed to read profile {}", path.display()))?;
        let profile: Profile = toml::from_str(&raw)
            .with_context(|| format!("failed to parse profile {}", path.display()))?;
        if profile.name != name {
            bail!(
                "profile file '{name}.toml' declares name = '{}'",
                profile.name
            );
        }
        Ok(profile)
    }

    pub fn resolve_profile(&self, name: &str) -> Result<ResolvedTheme> {
        let profile = self.load_profile(name)?;
        let palette = self
            .provider
            .load()
            .with_context(|| format!("provider '{}' failed", self.provider.id()))?;
        resolve::resolve(&profile, &palette)
    }

    fn render_targets(
        &self,
        theme: &ResolvedTheme,
        include_disabled: bool,
        disabled_targets: &BTreeSet<String>,
    ) -> Result<Vec<(&str, Vec<Artifact>)>> {
        let mut rendered = Vec::new();
        let mut owners = BTreeMap::new();

        for target in &self.targets {
            if !include_disabled && disabled_targets.contains(target.id()) {
                continue;
            }

            let artifacts = target
                .render(theme)
                .with_context(|| format!("target '{}' failed to render", target.id()))?;

            let declared = target
                .output_paths()
                .with_context(|| format!("target '{}' failed to declare outputs", target.id()))?
                .into_iter()
                .map(|path| canonical_output_path(&path))
                .collect::<Result<BTreeSet<_>>>()?;
            let rendered_paths = artifacts
                .iter()
                .map(|artifact| canonical_output_path(&artifact.path))
                .collect::<Result<BTreeSet<_>>>()?;
            if declared != rendered_paths {
                bail!(
                    "target '{}' rendered outputs that do not exactly match its declarations",
                    target.id()
                );
            }

            for artifact in &artifacts {
                let collision_key = canonical_output_path(&artifact.path).with_context(|| {
                    format!(
                        "failed to resolve target '{}' output {}",
                        target.id(),
                        artifact.path.display()
                    )
                })?;
                if let Some(previous) = owners.insert(collision_key, target.id()) {
                    bail!(
                        "targets '{previous}' and '{}' both write {}",
                        target.id(),
                        artifact.path.display(),
                    );
                }
            }

            rendered.push((target.id(), artifacts));
        }

        Ok(rendered)
    }

    pub fn apply(&self, name: &str, dry_run: bool) -> Result<ApplyReport> {
        self.apply_with_options(name, dry_run, false)
    }

    pub fn apply_with_options(
        &self,
        name: &str,
        dry_run: bool,
        force: bool,
    ) -> Result<ApplyReport> {
        self.apply_internal(name, dry_run, !dry_run, force)
    }

    pub fn reapply(&self, name: &str) -> Result<ApplyReport> {
        self.apply_internal(name, false, false, false)
    }

    fn apply_internal(
        &self,
        name: &str,
        dry_run: bool,
        make_active: bool,
        force: bool,
    ) -> Result<ApplyReport> {
        let state_file = self.config.state_file()?;
        state::with_lock(&state_file, || {
            transaction::recover(&state_file)?;
            let mut current_state = state::read(&state_file)?;
            let (report, operations) =
                self.apply_with_state(name, dry_run, force, &mut current_state)?;
            if dry_run {
                return Ok(report);
            }
            if make_active {
                current_state.active_profile = Some(name.to_string());
            }
            transaction::commit(&state_file, operations, &mut current_state)?;
            Ok(report)
        })
    }

    fn apply_with_state(
        &self,
        name: &str,
        dry_run: bool,
        force: bool,
        current_state: &mut state::State,
    ) -> Result<(ApplyReport, Vec<Operation>)> {
        let theme = self.resolve_profile(name)?;
        let rendered = self.render_targets(&theme, false, &current_state.disabled_targets)?;

        let mut reports = Vec::new();
        let mut operations = Vec::new();

        for (target, artifacts) in rendered {
            let mut changed = Vec::new();
            let mut unchanged = Vec::new();
            let mut forced = Vec::new();

            for artifact in artifacts {
                let ownership_hash = state::content_hash(artifact.contents.as_bytes());
                let ownership_key = canonical_output_path(&artifact.path)?
                    .to_string_lossy()
                    .into_owned();
                let did_change = contents_changed(&artifact.path, &artifact.contents)?;
                let existing = match fs::read(&artifact.path) {
                    Ok(contents) => Some(contents),
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
                    Err(error) => {
                        return Err(error).with_context(|| {
                            format!("failed to read output {}", artifact.path.display())
                        });
                    }
                };
                let existing_hash = existing.as_ref().map(state::content_hash);
                let mut was_forced = false;
                if let Some(contents) = &existing {
                    let current_hash = state::content_hash(contents);
                    let tracked = current_state
                        .generated_outputs
                        .get(&ownership_key)
                        .is_some_and(|output| {
                            output.target == target && output.hash == current_hash
                        });
                    let marker_owned = has_generated_marker(contents);
                    if !tracked && !marker_owned {
                        if !force {
                            bail!(
                                "refusing to replace {} because it has changed or is not owned by theme-manager; rerun the explicit apply with --force to replace it",
                                artifact.path.display()
                            );
                        }
                        was_forced = true;
                    }
                }
                if did_change {
                    if !dry_run {
                        operations.push(Operation::Write {
                            path: artifact.path.clone(),
                            contents: artifact.contents,
                            expected_hash: existing_hash,
                        });
                    }
                    if was_forced {
                        forced.push(artifact.path.clone());
                    }
                    changed.push(artifact.path.clone());
                } else {
                    unchanged.push(artifact.path.clone());
                }
                if !dry_run {
                    current_state.generated_outputs.insert(
                        ownership_key,
                        state::GeneratedOutput {
                            target: target.to_string(),
                            hash: ownership_hash,
                        },
                    );
                }
            }

            reports.push(TargetApplyReport {
                target: target.into(),
                changed,
                unchanged,
                forced,
            });
        }

        Ok((
            ApplyReport {
                profile: name.to_string(),
                dry_run,
                targets: reports,
            },
            operations,
        ))
    }

    pub fn validate_profile(&self, name: &str) -> Result<()> {
        let theme = self.resolve_profile(name)?;
        self.render_targets(&theme, true, &self.disabled_targets)?;
        Ok(())
    }

    pub fn profile_dir(&self) -> Result<PathBuf> {
        self.config.profiles_dir()
    }
    pub fn state_file(&self) -> Result<PathBuf> {
        self.config.state_file()
    }
}

fn has_generated_marker(contents: &[u8]) -> bool {
    let prefix = String::from_utf8_lossy(&contents[..contents.len().min(512)]);
    let header = prefix.lines().next();
    header == Some(GENERATED_MARKER)
        || header.and_then(|line| line.strip_prefix("# ")) == Some(GENERATED_MARKER)
        || header.and_then(|line| line.strip_prefix("// ")) == Some(GENERATED_MARKER)
}
