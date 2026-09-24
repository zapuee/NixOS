use anyhow::{Context, Result, bail};
use mlua::{Error as LuaError, Function, Lua, LuaSerdeExt, Table, Value, VmState, chunk::Compiler};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::{Component, Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};
use theme_core::{
    Artifact, ExtensionLimits, ExtensionsConfig, Palette, PaletteProvider, ResolvedTheme, Target,
    paths::expand_path,
};

const API_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ExtensionKind {
    Provider,
    Target,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    api: u32,
    id: String,
    kind: ExtensionKind,
    entry: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExtensionDescriptor {
    pub extension_id: String,
    pub adapter_id: String,
    pub kind: ExtensionKind,
    pub name: String,
    pub description: String,
    pub root: PathBuf,
    pub manifest_path: PathBuf,
    pub entry_path: PathBuf,
    #[serde(skip)]
    bytecode: Arc<Vec<u8>>,
    #[serde(skip)]
    limits: ExtensionLimits,
}

#[derive(Debug, Default)]
pub struct Discovery {
    pub extensions: Vec<ExtensionDescriptor>,
    pub diagnostics: Vec<String>,
    pub watch_paths: Vec<PathBuf>,
}

pub fn discover(config: &ExtensionsConfig) -> Result<Discovery> {
    config.validate()?;
    let mut discovery = Discovery::default();

    for directory in config.directories()? {
        discovery.watch_paths.push(directory.clone());
        if !directory.exists() {
            continue;
        }

        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(err) => {
                discovery.diagnostics.push(format!(
                    "failed to inspect extension directory {}: {err}",
                    directory.display()
                ));
                continue;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    discovery.diagnostics.push(format!(
                        "failed to inspect an entry in {}: {err}",
                        directory.display()
                    ));
                    continue;
                }
            };
            let root = entry.path();
            let manifest_path = root.join("plugin.toml");
            if !manifest_path.is_file() {
                continue;
            }

            match load_descriptor(&manifest_path, &config.limits) {
                Ok(descriptor) => discovery.extensions.push(descriptor),
                Err(err) => discovery
                    .diagnostics
                    .push(format!("{}: {err:#}", manifest_path.display())),
            }
        }
    }

    discovery
        .extensions
        .sort_by(|a, b| a.extension_id.cmp(&b.extension_id));
    discovery.watch_paths.sort();
    discovery.watch_paths.dedup();
    Ok(discovery)
}

fn load_descriptor(path: &Path, limits: &ExtensionLimits) -> Result<ExtensionDescriptor> {
    let raw = read_limited(path, limits.io_mib * 1024 * 1024)
        .with_context(|| format!("failed to read manifest {}", path.display()))?;
    let manifest: Manifest = toml::from_str(&raw)
        .with_context(|| format!("failed to parse manifest {}", path.display()))?;
    if manifest.api != API_VERSION {
        bail!(
            "unsupported extension API {}; expected {API_VERSION}",
            manifest.api
        );
    }
    validate_id(&manifest.id)?;

    let root = path
        .parent()
        .context("extension manifest has no parent directory")?
        .canonicalize()
        .context("failed to resolve extension directory")?;
    let entry_relative = safe_relative_path(&manifest.entry)?;
    if entry_relative.extension().and_then(|value| value.to_str()) != Some("luau") {
        bail!("extension entry must have a .luau suffix");
    }
    let entry_path = root
        .join(entry_relative)
        .canonicalize()
        .context("failed to resolve extension entry")?;
    ensure_inside(&entry_path, &root)?;
    let source = read_limited(&entry_path, limits.io_mib * 1024 * 1024)
        .with_context(|| format!("failed to read extension entry {}", entry_path.display()))?;
    let bytecode = Compiler::new()
        .set_optimization_level(1)
        .set_debug_level(1)
        .compile(source)
        .context("failed to compile Luau entry")?;

    Ok(ExtensionDescriptor {
        adapter_id: format!("luau:{}", manifest.id),
        extension_id: manifest.id,
        kind: manifest.kind,
        name: manifest.name,
        description: manifest.description,
        root,
        manifest_path: path.to_path_buf(),
        entry_path,
        bytecode: Arc::new(bytecode),
        limits: limits.clone(),
    })
}

fn validate_id(id: &str) -> Result<()> {
    if id.is_empty()
        || !id.chars().all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || matches!(character, '.' | '_' | '-')
        })
        || !id
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
    {
        bail!("invalid extension id '{id}'; use lowercase ASCII letters, digits, '.', '_' or '-'");
    }
    Ok(())
}

fn safe_relative_path(value: &str) -> Result<PathBuf> {
    let path = Path::new(value);
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        bail!("path '{value}' must be relative and may not contain traversal components");
    }
    Ok(path.to_path_buf())
}

fn ensure_inside(path: &Path, root: &Path) -> Result<()> {
    if !path.starts_with(root) {
        bail!(
            "path {} escapes extension root {}",
            path.display(),
            root.display()
        );
    }
    Ok(())
}

fn read_limited(path: &Path, maximum: usize) -> Result<String> {
    let metadata = fs::metadata(path)?;
    if !metadata.is_file() {
        bail!("{} is not a regular file", path.display());
    }
    let file = fs::File::open(path)?;
    let mut bytes = Vec::with_capacity((metadata.len() as usize).min(maximum));
    file.take(maximum.saturating_add(1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > maximum {
        bail!("{} exceeds the {maximum}-byte limit", path.display());
    }
    String::from_utf8(bytes).with_context(|| format!("{} is not valid UTF-8", path.display()))
}

fn empty_table() -> toml::Value {
    toml::Value::Table(Default::default())
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProviderConfig {
    #[serde(default)]
    inputs: BTreeMap<String, String>,
    #[serde(default = "empty_table")]
    settings: toml::Value,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TargetConfig {
    #[serde(default = "default_true")]
    enabled: bool,
    outputs: BTreeMap<String, String>,
    #[serde(default = "empty_table")]
    settings: toml::Value,
}

fn default_true() -> bool {
    true
}

#[derive(Clone)]
struct Host {
    descriptor: ExtensionDescriptor,
}

impl Host {
    fn new_vm(&self) -> Result<Lua> {
        let lua = Lua::new();
        lua.set_compiler(Compiler::new().set_optimization_level(1).set_debug_level(1));
        lua.set_memory_limit(self.descriptor.limits.memory_mib * 1024 * 1024)
            .context("failed to set Luau memory limit")?;

        let deadline = Instant::now() + Duration::from_millis(self.descriptor.limits.execution_ms);
        lua.set_interrupt(move |_| {
            if Instant::now() >= deadline {
                Err(LuaError::runtime("extension execution deadline exceeded"))
            } else {
                Ok(VmState::Continue)
            }
        });

        install_require(
            &lua,
            self.descriptor.root.clone(),
            self.descriptor.limits.io_mib * 1024 * 1024,
        )?;
        lua.sandbox(true).context("failed to enable Luau sandbox")?;
        Ok(lua)
    }

    fn module(&self, lua: &Lua) -> Result<Table> {
        lua.load(self.descriptor.bytecode.as_slice())
            .set_name(format!("@{}", self.descriptor.entry_path.display()))
            .eval::<Table>()
            .with_context(|| {
                format!(
                    "extension '{}' entry must return a table",
                    self.descriptor.adapter_id
                )
            })
    }
}

pub fn validate_extension(descriptor: &ExtensionDescriptor) -> Result<()> {
    let host = Host {
        descriptor: descriptor.clone(),
    };
    let lua = host.new_vm()?;
    let module = host.module(&lua)?;
    let export = match descriptor.kind {
        ExtensionKind::Provider => "load",
        ExtensionKind::Target => "render",
    };
    let _: Function = module.get(export).with_context(|| {
        format!(
            "extension '{}' must export {export}(ctx)",
            descriptor.adapter_id
        )
    })?;
    Ok(())
}

fn install_require(lua: &Lua, root: PathBuf, maximum: usize) -> mlua::Result<()> {
    let cache = lua.create_table()?;
    let loading = lua.create_table()?;
    let require = lua.create_function(move |lua, request: String| {
        let requested = request.strip_prefix("./").ok_or_else(|| {
            LuaError::runtime("require paths must begin with './' and stay inside the plugin")
        })?;
        let mut relative = safe_relative_path(requested).map_err(LuaError::external)?;
        if relative.extension().is_none() {
            relative.set_extension("luau");
        }
        if relative.extension().and_then(|value| value.to_str()) != Some("luau") {
            return Err(LuaError::runtime(
                "required modules must use the .luau suffix",
            ));
        }

        let path = root
            .join(relative)
            .canonicalize()
            .map_err(LuaError::external)?;
        ensure_inside(&path, &root).map_err(LuaError::external)?;
        let key = path.to_string_lossy().into_owned();
        let cached: Value = cache.raw_get(key.as_str())?;
        if !matches!(cached, Value::Nil) {
            return Ok(cached);
        }
        if loading.raw_get::<bool>(key.as_str()).unwrap_or(false) {
            return Err(LuaError::runtime(format!(
                "cyclic require involving {}",
                path.display()
            )));
        }

        loading.raw_set(key.as_str(), true)?;
        let result = (|| {
            let source = read_limited(&path, maximum).map_err(LuaError::external)?;
            let value = lua
                .load(source)
                .set_name(format!("@{}", path.display()))
                .eval::<Value>()?;
            if matches!(value, Value::Nil) {
                return Err(LuaError::runtime(format!(
                    "module {} returned nil",
                    path.display()
                )));
            }
            cache.raw_set(key.as_str(), value.clone())?;
            Ok(value)
        })();
        loading.raw_set(key.as_str(), Value::Nil)?;
        result
    })?;
    lua.globals().raw_set("require", require)
}

fn install_context_helpers(lua: &Lua, context: &Table) -> mlua::Result<()> {
    let json = lua.create_table()?;
    json.set(
        "decode",
        lua.create_function(|lua, input: String| {
            let value: JsonValue = serde_json::from_str(&input).map_err(LuaError::external)?;
            lua.to_value(&value)
        })?,
    )?;
    json.set(
        "encode",
        lua.create_function(|lua, input: Value| {
            let value: JsonValue = lua.from_value(input)?;
            serde_json::to_string(&value).map_err(LuaError::external)
        })?,
    )?;
    context.set("json", json)
}

pub struct LuauProvider {
    host: Host,
    inputs: BTreeMap<String, PathBuf>,
    settings: toml::Value,
}

impl LuauProvider {
    fn from_toml(descriptor: ExtensionDescriptor, value: &toml::Value) -> Result<Self> {
        if descriptor.kind != ExtensionKind::Provider {
            bail!("'{}' is not a provider extension", descriptor.adapter_id);
        }
        let config: ProviderConfig = value
            .clone()
            .try_into()
            .context("invalid Luau provider configuration")?;
        let inputs = config
            .inputs
            .into_iter()
            .map(|(name, path)| Ok((name, expand_path(&path)?)))
            .collect::<Result<_>>()?;
        Ok(Self {
            host: Host { descriptor },
            inputs,
            settings: config.settings,
        })
    }
}

impl PaletteProvider for LuauProvider {
    fn id(&self) -> &str {
        &self.host.descriptor.adapter_id
    }

    fn load(&self) -> Result<Palette> {
        let lua = self.host.new_vm()?;
        let module = self.host.module(&lua)?;
        let load: Function = module
            .get("load")
            .context("provider extension must export load(ctx)")?;
        let context = lua.create_table()?;
        install_context_helpers(&lua, &context)?;
        context.set("settings", lua.to_value(&self.settings)?)?;
        let inputs = lua.create_table()?;
        let maximum = self.host.descriptor.limits.io_mib * 1024 * 1024;
        let mut total = 0usize;
        for (name, path) in &self.inputs {
            let contents = read_limited(path, maximum)
                .with_context(|| format!("failed to read provider input '{name}'"))?;
            total = total
                .checked_add(contents.len())
                .context("provider input size overflow")?;
            if total > maximum {
                bail!("provider inputs exceed the combined {maximum}-byte limit");
            }
            inputs.set(name.as_str(), contents)?;
        }
        context.set("inputs", inputs)?;

        let colors = lua
            .from_value::<BTreeMap<String, String>>(load.call::<Value>(context)?)
            .context("provider load(ctx) must return a string-to-string palette table")?;
        let palette = Palette { colors };
        palette.validate()?;
        Ok(palette)
    }

    fn watch_paths(&self) -> Vec<PathBuf> {
        self.inputs
            .values()
            .cloned()
            .chain([self.host.descriptor.root.clone()])
            .collect()
    }
}

pub struct LuauTarget {
    host: Host,
    outputs: BTreeMap<String, PathBuf>,
    settings: toml::Value,
}

impl LuauTarget {
    fn from_toml(descriptor: ExtensionDescriptor, value: &toml::Value) -> Result<Option<Self>> {
        if descriptor.kind != ExtensionKind::Target {
            bail!("'{}' is not a target extension", descriptor.adapter_id);
        }
        let config: TargetConfig = value
            .clone()
            .try_into()
            .context("invalid Luau target configuration")?;
        if !config.enabled {
            return Ok(None);
        }
        if config.outputs.is_empty() {
            bail!("Luau targets must declare at least one named output");
        }
        let outputs = config
            .outputs
            .into_iter()
            .map(|(name, path)| Ok((name, expand_path(&path)?)))
            .collect::<Result<_>>()?;
        Ok(Some(Self {
            host: Host { descriptor },
            outputs,
            settings: config.settings,
        }))
    }
}

impl Target for LuauTarget {
    fn id(&self) -> &str {
        &self.host.descriptor.adapter_id
    }

    fn output_paths(&self) -> Result<Vec<PathBuf>> {
        Ok(self.outputs.values().cloned().collect())
    }

    fn render(&self, theme: &ResolvedTheme) -> Result<Vec<Artifact>> {
        let lua = self.host.new_vm()?;
        let module = self.host.module(&lua)?;
        let render: Function = module
            .get("render")
            .context("target extension must export render(ctx)")?;
        let context = lua.create_table()?;
        install_context_helpers(&lua, &context)?;
        context.set("settings", lua.to_value(&self.settings)?)?;
        context.set("theme", lua.to_value(theme)?)?;
        let rendered = lua
            .from_value::<BTreeMap<String, String>>(render.call::<Value>(context)?)
            .context("target render(ctx) must return a string-to-string output table")?;

        let expected = self.outputs.keys().cloned().collect::<BTreeSet<_>>();
        let actual = rendered.keys().cloned().collect::<BTreeSet<_>>();
        if expected != actual {
            let missing = expected.difference(&actual).cloned().collect::<Vec<_>>();
            let unexpected = actual.difference(&expected).cloned().collect::<Vec<_>>();
            bail!(
                "target output mismatch; missing: [{}], unexpected: [{}]",
                missing.join(", "),
                unexpected.join(", ")
            );
        }

        let maximum = self.host.descriptor.limits.io_mib * 1024 * 1024;
        let total = rendered.values().try_fold(0usize, |total, contents| {
            total
                .checked_add(contents.len())
                .context("output size overflow")
        })?;
        if total > maximum {
            bail!("target outputs exceed the combined {maximum}-byte limit");
        }

        Ok(rendered
            .into_iter()
            .map(|(name, contents)| Artifact {
                path: self.outputs[&name].clone(),
                contents,
            })
            .collect())
    }
}

pub fn build_provider(
    descriptor: ExtensionDescriptor,
    value: &toml::Value,
) -> Result<Box<dyn PaletteProvider>> {
    Ok(Box::new(LuauProvider::from_toml(descriptor, value)?))
}

pub fn build_target(
    descriptor: ExtensionDescriptor,
    value: &toml::Value,
) -> Result<Option<Box<dyn Target>>> {
    Ok(LuauTarget::from_toml(descriptor, value)?.map(|target| Box::new(target) as Box<dyn Target>))
}

#[cfg(test)]
mod tests {
    use super::{
        build_provider, build_target, load_descriptor, safe_relative_path, validate_extension,
        validate_id,
    };
    use std::{
        collections::BTreeMap,
        fs,
        path::PathBuf,
        sync::atomic::{AtomicUsize, Ordering},
    };
    use theme_core::{
        ExtensionLimits, PaletteProvider, Target,
        profile::{ResolvedFocusRing, ResolvedLayout, ResolvedShadow, ResolvedTheme},
    };

    static NEXT_FIXTURE: AtomicUsize = AtomicUsize::new(0);

    fn fixture(source: &str) -> (PathBuf, super::ExtensionDescriptor) {
        let index = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "theme-manager-luau-test-{}-{index}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("plugin.toml"),
            "api = 1\nid = \"test\"\nkind = \"provider\"\nentry = \"main.luau\"\n",
        )
        .unwrap();
        fs::write(root.join("main.luau"), source).unwrap();
        let descriptor =
            load_descriptor(&root.join("plugin.toml"), &ExtensionLimits::default()).unwrap();
        (root, descriptor)
    }

    #[test]
    fn extension_ids_are_namespaced_and_predictable() {
        assert!(validate_id("generic-renderer_1").is_ok());
        assert!(validate_id("Bad").is_err());
        assert!(validate_id("-bad").is_err());
    }

    #[test]
    fn manifest_paths_cannot_escape() {
        assert!(safe_relative_path("main.luau").is_ok());
        assert!(safe_relative_path("modules/colors.luau").is_ok());
        assert!(safe_relative_path("../escape.luau").is_err());
        assert!(safe_relative_path("/absolute.luau").is_err());
    }

    #[test]
    fn extension_validation_requires_the_kind_specific_export() {
        let (root, descriptor) = fixture("return {}\n");
        let error = validate_extension(&descriptor).unwrap_err().to_string();
        assert!(error.contains("must export load(ctx)"), "{error}");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn provider_runs_in_a_sandbox_with_confined_modules() {
        let (root, descriptor) = fixture(
            r##"
local palette = require("./palette")
return {
    load = function(ctx)
        assert(io == nil)
        assert(loadfile == nil)
        assert(os.execute == nil)
        assert(ctx.settings.variant == "dark")
        return palette
    end,
}
"##,
        );
        fs::write(
            root.join("palette.luau"),
            "return { primary = \"#ffffff\", surface = \"#000000\" }\n",
        )
        .unwrap();
        let config: toml::Value = toml::from_str(
            r#"
[settings]
variant = "dark"
"#,
        )
        .unwrap();
        let provider: Box<dyn PaletteProvider> = build_provider(descriptor, &config).unwrap();
        let palette = provider.load().unwrap();
        assert_eq!(palette.colors["primary"], "#ffffff");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn provider_can_decode_host_read_json_without_filesystem_access() {
        let (root, descriptor) = fixture(
            r#"
return {
    load = function(ctx)
        local decoded = ctx.json.decode(ctx.inputs.palette)
        assert(io == nil)
        return decoded
    end,
}
"#,
        );
        let input = root.join("palette.json");
        fs::write(&input, r##"{"primary":"#ffffff","surface":"#000000"}"##).unwrap();
        let config: toml::Value = toml::from_str(&format!(
            "[inputs]\npalette = {:?}\n",
            input.to_string_lossy()
        ))
        .unwrap();
        let provider = build_provider(descriptor, &config).unwrap();
        let palette = provider.load().unwrap();
        assert_eq!(palette.colors["surface"], "#000000");
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn required_module_symlinks_cannot_escape_the_plugin() {
        use std::os::unix::fs::symlink;

        let (root, descriptor) = fixture(
            r#"
local palette = require("./escape")
return { load = function(_ctx) return palette end }
"#,
        );
        let outside = root.with_extension("outside.luau");
        fs::write(&outside, "return { primary = \"#ffffff\" }\n").unwrap();
        symlink(&outside, root.join("escape.luau")).unwrap();

        let config: toml::Value = toml::from_str("").unwrap();
        let provider = build_provider(descriptor, &config).unwrap();
        let error = format!("{:#}", provider.load().unwrap_err());
        assert!(error.contains("escapes extension root"), "{error}");

        fs::remove_dir_all(root).unwrap();
        fs::remove_file(outside).unwrap();
    }

    #[test]
    fn runaway_extension_is_interrupted() {
        let (root, mut descriptor) =
            fixture("return { load = function(_ctx) while true do end end }\n");
        descriptor.limits.execution_ms = 10;
        let config: toml::Value = toml::from_str("").unwrap();
        let provider = build_provider(descriptor, &config).unwrap();
        let error = provider.load().unwrap_err().to_string();
        assert!(error.contains("deadline"), "{error}");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn target_is_application_neutral_and_cannot_write_directly() {
        let (root, mut descriptor) = fixture(
            r##"
return {
    render = function(ctx)
        return { document = "{\"profile\":\"" .. ctx.theme.profile .. "\"}\n" }
    end,
}
"##,
        );
        descriptor.kind = super::ExtensionKind::Target;
        let output = root.join("generated.json");
        let config: toml::Value = toml::from_str(&format!(
            "[outputs]\ndocument = {:?}\n",
            output.to_string_lossy()
        ))
        .unwrap();
        let target: Box<dyn Target> = build_target(descriptor, &config).unwrap().unwrap();
        let theme = ResolvedTheme {
            schema: 1,
            profile: "example".into(),
            description: String::new(),
            layout: ResolvedLayout {
                gaps: 0.0,
                outer_gaps: 0.0,
            },
            focus_ring: ResolvedFocusRing {
                width: 0.0,
                active_opacity: 1.0,
                inactive_opacity: 1.0,
                urgent_opacity: 1.0,
                active_color: "#ffffff".into(),
                inactive_color: "#ffffff".into(),
                urgent_color: "#ffffff".into(),
            },
            shadow: ResolvedShadow {
                strength: 0.0,
                color: "#000000".into(),
            },
            roles: BTreeMap::new(),
        };
        let artifacts = target.render(&theme).unwrap();
        assert_eq!(artifacts[0].contents, "{\"profile\":\"example\"}\n");
        assert!(!output.exists(), "only Rust may write returned artifacts");
        fs::remove_dir_all(root).unwrap();
    }
}
