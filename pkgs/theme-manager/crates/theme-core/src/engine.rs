use crate::{
    Artifact, Config, PaletteProvider, Profile, ResolvedTheme, Target,
    fs::{contents_changed, write_if_changed},
    paths::canonical_output_path,
    resolve, state,
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
        self.apply_internal(name, dry_run, !dry_run)
    }

    pub fn reapply(&self, name: &str) -> Result<ApplyReport> {
        self.apply_internal(name, false, false)
    }

    fn apply_internal(&self, name: &str, dry_run: bool, make_active: bool) -> Result<ApplyReport> {
        let state_file = self.config.state_file()?;
        if dry_run {
            let mut current_state = state::read(&state_file)?;
            return self.apply_with_state(name, true, &mut current_state);
        }

        state::with_lock(&state_file, || {
            let mut current_state = state::read(&state_file)?;
            let report = match self.apply_with_state(name, false, &mut current_state) {
                Ok(report) => report,
                Err(err) => {
                    // Earlier artifacts may already have been atomically
                    // replaced. Persist their ownership records so a later
                    // cleanup never mistakes them for unmanaged files.
                    state::write(&state_file, &current_state).with_context(|| {
                        format!(
                            "apply failed ({err:#}) and updated output ownership could not be persisted"
                        )
                    })?;
                    return Err(err);
                }
            };
            if make_active {
                current_state.active_profile = Some(name.to_string());
            }
            state::write(&state_file, &current_state)?;
            Ok(report)
        })
    }

    fn apply_with_state(
        &self,
        name: &str,
        dry_run: bool,
        current_state: &mut state::State,
    ) -> Result<ApplyReport> {
        let theme = self.resolve_profile(name)?;
        let rendered = self.render_targets(&theme, false, &current_state.disabled_targets)?;

        let mut reports = Vec::new();

        for (target, artifacts) in rendered {
            let mut changed = Vec::new();
            let mut unchanged = Vec::new();

            for artifact in artifacts {
                let ownership_hash = state::content_hash(artifact.contents.as_bytes());
                let ownership_key = if dry_run {
                    None
                } else {
                    Some(
                        canonical_output_path(&artifact.path)?
                            .to_string_lossy()
                            .into_owned(),
                    )
                };
                let did_change = if dry_run {
                    contents_changed(&artifact.path, &artifact.contents)?
                } else {
                    write_if_changed(&artifact.path, &artifact.contents)?
                };

                if did_change {
                    changed.push(artifact.path);
                } else {
                    unchanged.push(artifact.path);
                }
                if let Some(ownership_key) = ownership_key {
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
            });
        }

        Ok(ApplyReport {
            profile: name.to_string(),
            dry_run,
            targets: reports,
        })
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
