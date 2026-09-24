use crate::registry::Registry;
use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::{
    collections::BTreeMap,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};
use theme_core::{Config, Engine, GENERATED_MARKER, paths::canonical_output_path, state};

#[derive(Debug, Clone, Serialize)]
pub struct TargetStatus {
    pub target: String,
    pub adapter: String,
    pub configured: bool,
    pub toggleable: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TargetChangeReport {
    pub target: String,
    pub enabled: bool,
    pub changed: Vec<PathBuf>,
    pub removed: Vec<PathBuf>,
}

struct TargetControl {
    status: TargetStatus,
    output_paths: Vec<PathBuf>,
}

pub struct Runtime {
    pub engine: Engine,
    pub config_path: PathBuf,
    target_controls: BTreeMap<String, TargetControl>,
    extension_watch_paths: Vec<PathBuf>,
}

impl Runtime {
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let (config, config_path) = Config::load(path)?;
        let registry = Registry::load(&config)?;

        let provider_settings = config.provider.settings_value();
        let provider = registry
            .build_provider(&config.provider.kind, &provider_settings)
            .with_context(|| format!("failed to initialize provider '{}'", config.provider.kind))?;

        let state_file = config.state_file()?;
        let current_state = state::read(&state_file)?;
        let mut target_controls = BTreeMap::new();
        let mut targets = Vec::new();
        let mut output_owners = BTreeMap::new();
        for (instance, settings) in &config.targets {
            let (adapter, adapter_settings) = target_adapter(instance, settings)?;
            let target = registry
                .build_target(instance, &adapter, &adapter_settings)
                .with_context(|| {
                    format!("failed to initialize target '{instance}' with adapter '{adapter}'")
                })?;
            let control =
                target_controls
                    .entry(instance.clone())
                    .or_insert_with(|| TargetControl {
                        status: TargetStatus {
                            target: instance.clone(),
                            adapter: adapter.clone(),
                            configured: false,
                            toggleable: false,
                            enabled: false,
                        },
                        output_paths: Vec::new(),
                    });
            control.status.adapter.clone_from(&adapter);
            control.status.configured = true;

            if let Some(target) = target {
                control.output_paths = target.output_paths().with_context(|| {
                    format!("failed to resolve outputs for target '{instance}'")
                })?;
                for output in &control.output_paths {
                    let collision_key = canonical_output_path(output).with_context(|| {
                        format!(
                            "failed to resolve output for target '{instance}': {}",
                            output.display()
                        )
                    })?;
                    if let Some(previous) = output_owners.insert(collision_key, instance.as_str()) {
                        bail!(
                            "targets '{previous}' and '{instance}' both write {}",
                            output.display()
                        );
                    }
                }
                control.status.toggleable = true;
                control.status.enabled = !current_state.disabled_targets.contains(instance);
                targets.push(target);
            }
        }

        let extension_watch_paths = registry.extension_watch_paths().to_vec();

        let engine = Engine::new(config, provider, targets)
            .with_disabled_targets(current_state.disabled_targets);

        Ok(Self {
            engine,
            config_path,
            target_controls,
            extension_watch_paths,
        })
    }

    pub fn target_statuses(&self) -> Vec<TargetStatus> {
        self.target_controls
            .values()
            .map(|control| control.status.clone())
            .collect()
    }

    pub fn set_target_enabled(&self, target: &str, enabled: bool) -> Result<TargetChangeReport> {
        let control = self
            .target_controls
            .get(target)
            .ok_or_else(|| anyhow::anyhow!("unknown target '{target}'"))?;
        if !control.status.configured {
            bail!("target '{target}' is not configured");
        }
        if !control.status.toggleable {
            bail!(
                "target '{target}' is disabled in config; set enabled = true before using runtime controls"
            );
        }

        let mut report = TargetChangeReport {
            target: target.to_string(),
            enabled,
            changed: Vec::new(),
            removed: Vec::new(),
        };
        let state_file = self.engine.state_file()?;
        let changed = state::with_lock(&state_file, || {
            let mut current_state = state::read(&state_file)?;
            let currently_enabled = !current_state.disabled_targets.contains(target);
            if currently_enabled == enabled {
                return Ok(false);
            }

            if !enabled {
                match remove_outputs(&control.output_paths, target, &mut current_state) {
                    Ok(removed) => report.removed = removed,
                    Err(err) => {
                        state::write(&state_file, &current_state)?;
                        return Err(err);
                    }
                }
            }

            if enabled {
                current_state.disabled_targets.remove(target);
            } else {
                current_state.disabled_targets.insert(target.to_string());
            }
            state::write(&state_file, &current_state)?;
            Ok(true)
        })?;
        if !changed {
            return Ok(report);
        }

        if enabled {
            let refreshed = Self::load(Some(&self.config_path))?;
            if let Some(profile) = refreshed.engine.active_profile()? {
                match refreshed.engine.reapply(&profile) {
                    Ok(apply) => {
                        if let Some(target_report) = apply
                            .targets
                            .into_iter()
                            .find(|candidate| candidate.target == target)
                        {
                            report.changed = target_report.changed;
                        }
                    }
                    Err(err) => {
                        let rollback = state::with_lock(&state_file, || {
                            let mut current_state = state::read(&state_file)?;
                            current_state.disabled_targets.insert(target.to_string());
                            let cleanup =
                                remove_outputs(&control.output_paths, target, &mut current_state);
                            state::write(&state_file, &current_state)?;
                            cleanup.map(|_| ())
                        });
                        if let Err(rollback_err) = rollback {
                            return Err(rollback_err).with_context(|| {
                                format!(
                                    "failed to enable target '{target}' ({err:#}); rollback was incomplete"
                                )
                            });
                        }
                        return Err(err).with_context(|| {
                            format!(
                                "failed to enable target '{target}'; the change was rolled back"
                            )
                        });
                    }
                }
            }
        }

        Ok(report)
    }

    pub fn watch_paths(&self) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();

        add_watch_locations(&mut paths, &self.config_path);

        let profile_dir = self.engine.profile_dir()?;
        add_watch_locations(&mut paths, &profile_dir);
        if profile_dir.is_dir() {
            for entry in fs::read_dir(&profile_dir).with_context(|| {
                format!("failed to inspect profiles in {}", profile_dir.display())
            })? {
                let path = entry?.path();
                if path.extension().and_then(|value| value.to_str()) == Some("toml") {
                    add_watch_locations(&mut paths, &path);
                }
            }
        }

        add_watch_locations(&mut paths, &self.engine.state_file()?);

        for provider_path in self.engine.provider_watch_paths() {
            add_watch_locations(&mut paths, &provider_path);
        }

        for extension_path in &self.extension_watch_paths {
            add_watch_locations(&mut paths, extension_path);
            add_child_directories(&mut paths, extension_path)?;
        }

        paths.sort();
        paths.dedup();
        Ok(paths)
    }
}

fn add_child_directories(paths: &mut Vec<PathBuf>, root: &Path) -> Result<()> {
    if !root.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(root)
        .with_context(|| format!("failed to inspect extension directory {}", root.display()))?
    {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_dir() && !file_type.is_symlink() {
            let path = entry.path();
            add_watch_locations(paths, &path);
            add_child_directories(paths, &path)?;
        }
    }
    Ok(())
}

fn target_adapter(instance: &str, settings: &toml::Value) -> Result<(String, toml::Value)> {
    let mut table = settings
        .as_table()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("target '{instance}' configuration must be a table"))?;
    let adapter = match table.remove("adapter") {
        Some(toml::Value::String(adapter)) if !adapter.is_empty() => adapter,
        Some(_) => bail!("target '{instance}' adapter must be a non-empty string"),
        None => instance.to_string(),
    };
    Ok((adapter, toml::Value::Table(table)))
}

fn add_watch_locations(paths: &mut Vec<PathBuf>, path: &Path) {
    let logical = if path.is_dir() {
        path.to_path_buf()
    } else {
        parent_or_current(path)
    };
    paths.push(logical.clone());

    if let Ok(canonical) = fs::canonicalize(&logical) {
        paths.push(canonical);
    }
    if let Ok(canonical) = fs::canonicalize(path) {
        paths.push(if canonical.is_dir() {
            canonical
        } else {
            parent_or_current(&canonical)
        });
    }
}

fn remove_outputs(
    paths: &[PathBuf],
    target: &str,
    current_state: &mut state::State,
) -> Result<Vec<PathBuf>> {
    let mut owned = Vec::new();

    for path in paths {
        match fs::metadata(path) {
            Ok(metadata) if !metadata.is_file() => {
                bail!(
                    "refusing to remove {} because it is not a regular file",
                    path.display()
                )
            }
            Ok(_) => {}
            Err(err) if err.kind() == ErrorKind::NotFound => {
                if let Ok(key) = canonical_output_path(path) {
                    let key = key.to_string_lossy().into_owned();
                    if current_state
                        .generated_outputs
                        .get(&key)
                        .is_some_and(|output| output.target == target)
                    {
                        current_state.generated_outputs.remove(&key);
                    }
                }
                continue;
            }
            Err(err) => {
                return Err(err)
                    .with_context(|| format!("failed to inspect output {}", path.display()));
            }
        }
        match fs::read(path) {
            Ok(contents) => {
                let ownership_key = canonical_output_path(path)?.to_string_lossy().into_owned();
                let prefix = String::from_utf8_lossy(&contents[..contents.len().min(512)]);
                let header = prefix.lines().next();
                let marker_owned = header == Some(GENERATED_MARKER)
                    || header.and_then(|line| line.strip_prefix("# ")) == Some(GENERATED_MARKER)
                    || header.and_then(|line| line.strip_prefix("// ")) == Some(GENERATED_MARKER);
                let is_owned = match current_state.generated_outputs.get(&ownership_key) {
                    Some(output) => {
                        output.target == target && output.hash == state::content_hash(&contents)
                    }
                    None => marker_owned,
                };
                if !is_owned {
                    bail!(
                        "refusing to remove {} because it has changed or is not owned by theme-manager",
                        path.display()
                    );
                }
                owned.push((
                    path.to_path_buf(),
                    ownership_key,
                    state::content_hash(&contents),
                ));
            }
            Err(err) if err.kind() == ErrorKind::NotFound => continue,
            Err(err) => {
                return Err(err)
                    .with_context(|| format!("failed to inspect output {}", path.display()));
            }
        }
    }

    // Recheck every file immediately before deletion. This preserves the
    // all-or-nothing preflight if an editor changed an output during the first
    // validation pass.
    for (path, _, expected_hash) in &owned {
        let contents = fs::read(path)
            .with_context(|| format!("failed to recheck output {}", path.display()))?;
        if state::content_hash(&contents) != *expected_hash {
            bail!(
                "refusing to remove {} because it changed during validation",
                path.display()
            );
        }
    }

    let mut removed = Vec::new();
    for (path, ownership_key, _) in owned {
        match fs::remove_file(&path) {
            Ok(()) => {
                current_state.generated_outputs.remove(&ownership_key);
                removed.push(path);
            }
            Err(err) if err.kind() == ErrorKind::NotFound => {
                current_state.generated_outputs.remove(&ownership_key);
            }
            Err(err) => {
                return Err(err).with_context(|| {
                    format!("failed to remove generated output {}", path.display())
                });
            }
        }
    }

    Ok(removed)
}

pub fn parent_or_current(path: &Path) -> PathBuf {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::{Runtime, add_watch_locations, remove_outputs};
    use std::{
        fs,
        path::Path,
        sync::atomic::{AtomicUsize, Ordering},
    };
    use theme_core::{GENERATED_MARKER, state::State};

    static NEXT_FIXTURE: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn output_removal_checks_every_file_before_deleting_any() {
        let id = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "theme-manager-runtime-test-{}-{id}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        let owned = root.join("owned.conf");
        let unmanaged = root.join("unmanaged.conf");
        fs::write(&owned, format!("# {GENERATED_MARKER}\n")).unwrap();
        fs::write(&unmanaged, "# user configuration\n").unwrap();

        assert!(
            remove_outputs(&[owned.clone(), unmanaged], "foot", &mut State::default()).is_err()
        );
        assert!(owned.exists());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn stale_runtime_cannot_recreate_a_disabled_target() {
        let id = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "theme-manager-runtime-test-{}-{id}",
            std::process::id()
        ));
        let profiles = root.join("profiles");
        fs::create_dir_all(&profiles).unwrap();
        fs::write(
            profiles.join("glass.toml"),
            include_str!("../../../examples/profiles/glass.toml"),
        )
        .unwrap();
        let output = root.join("foot.ini");
        let state = root.join("state.toml");
        let config = root.join("config.toml");
        let plugins = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../plugins");
        fs::write(
            &config,
            format!(
                r##"profiles_dir = "{}"
state_file = "{}"

[extensions]
dirs = ["{}"]

[provider]
kind = "luau:static"

[provider.settings.colors]
primary = "#80d998"
outline = "#7c9598"
error = "#ffb4ab"
shadow = "#000000"
secondary = "#9ccaff"
surface = "#0f1512"
on_surface = "#dfe4de"

[targets.foot]
adapter = "luau:foot"
[targets.foot.outputs]
config = "{}"
[targets.foot.settings]
role = "terminal"
"##,
                profiles.display(),
                state.display(),
                plugins.display(),
                output.display(),
            ),
        )
        .unwrap();

        let stale = Runtime::load(Some(&config)).unwrap();
        stale.engine.apply("glass", false).unwrap();
        let controller = Runtime::load(Some(&config)).unwrap();
        controller.set_target_enabled("foot", false).unwrap();
        stale.engine.reapply("glass").unwrap();
        assert!(!output.exists());

        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn watch_locations_follow_file_symlinks() {
        use std::os::unix::fs::symlink;

        let id = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "theme-manager-runtime-test-{}-{id}",
            std::process::id()
        ));
        let source_dir = root.join("source");
        let link_dir = root.join("links");
        fs::create_dir_all(&source_dir).unwrap();
        fs::create_dir_all(&link_dir).unwrap();
        let source = source_dir.join("profile.toml");
        let link = link_dir.join("profile.toml");
        fs::write(&source, "name = \"test\"\n").unwrap();
        symlink(&source, &link).unwrap();

        let mut paths = Vec::new();
        add_watch_locations(&mut paths, &link);
        assert!(paths.contains(&fs::canonicalize(&source_dir).unwrap()));
        assert!(paths.contains(&fs::canonicalize(&link_dir).unwrap()));

        fs::remove_dir_all(root).unwrap();
    }
}
