use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
    thread,
    time::{Duration, Instant},
};

static NEXT_FIXTURE: AtomicUsize = AtomicUsize::new(0);

struct Fixture {
    root: PathBuf,
}

struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

impl Fixture {
    fn new() -> Self {
        let id = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("theme-manager-test-{}-{id}", std::process::id()));
        let profiles = root.join("profiles");
        fs::create_dir_all(&profiles).unwrap();
        fs::write(
            profiles.join("glass.toml"),
            include_str!("../../../examples/profiles/glass.toml"),
        )
        .unwrap();
        Self { root }
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    fn write_config(&self, targets: &str) -> PathBuf {
        let path = self.path("config.toml");
        let profiles = self.path("profiles");
        let state = self.path("state.toml");
        let config = format!(
            r##"
profiles_dir = "{}"
state_file = "{}"
default_profile = "glass"

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

{targets}
"##,
            profiles.display(),
            state.display(),
        );
        fs::write(&path, config).unwrap();
        path
    }

    fn run(&self, config: &Path, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_theme-manager"))
            .env(
                "THEME_MANAGER_DATA_DIR",
                Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."),
            )
            .arg("--config")
            .arg(config)
            .args(args)
            .output()
            .unwrap()
    }
}

#[test]
fn apply_refuses_unmanaged_output_unless_force_is_explicit() {
    let fixture = Fixture::new();
    let output = fixture.path("foot.ini");
    fs::write(&output, "user-owned configuration\n").unwrap();
    let config = fixture.write_config(&format!(
        r#"[targets.foot]
adapter = "luau:foot"
[targets.foot.outputs]
config = "{}"
[targets.foot.settings]
role = "terminal""#,
        output.display(),
    ));

    let refused = fixture.run(&config, &["apply", "glass"]);
    assert!(!refused.status.success());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("--force"));
    assert_eq!(
        fs::read_to_string(&output).unwrap(),
        "user-owned configuration\n"
    );

    let forced = fixture.run(&config, &["apply", "glass", "--force", "--json"]);
    assert!(
        forced.status.success(),
        "{}",
        String::from_utf8_lossy(&forced.stderr)
    );
    let report: Value = serde_json::from_slice(&forced.stdout).unwrap();
    assert_eq!(
        report["targets"][0]["forced"][0],
        output.display().to_string()
    );
    assert!(fs::read_to_string(output).unwrap().contains("alpha=0.25"));
}

#[test]
fn apply_refuses_a_modified_owned_output() {
    let fixture = Fixture::new();
    let output = fixture.path("foot.ini");
    let config = fixture.write_config(&format!(
        r#"[targets.foot]
adapter = "luau:foot"
[targets.foot.outputs]
config = "{}"
[targets.foot.settings]
role = "terminal""#,
        output.display(),
    ));

    assert!(fixture.run(&config, &["apply", "glass"]).status.success());
    fs::write(&output, "manually edited\n").unwrap();
    let refused = fixture.run(&config, &["apply", "glass"]);
    assert!(!refused.status.success());
    assert_eq!(fs::read_to_string(output).unwrap(), "manually edited\n");
}

#[test]
fn output_preflight_prevents_partial_writes_on_filesystem_errors() {
    let fixture = Fixture::new();
    let first = fixture.path("a-foot.ini");
    let invalid = fixture.path("z-invalid");
    fs::create_dir_all(&invalid).unwrap();
    let config = fixture.write_config(&format!(
        r#"[targets.first]
adapter = "luau:foot"
[targets.first.outputs]
config = "{}"
[targets.first.settings]
role = "terminal"

[targets.second]
adapter = "luau:foot"
[targets.second.outputs]
config = "{}"
[targets.second.settings]
role = "terminal""#,
        first.display(),
        invalid.display(),
    ));

    let apply = fixture.run(&config, &["apply", "glass"]);
    assert!(!apply.status.success());
    assert!(!first.exists());
    assert!(invalid.is_dir());
}

#[test]
fn doctor_has_a_versioned_json_report_and_warnings_do_not_fail() {
    let fixture = Fixture::new();
    let config = fixture.write_config("");
    let doctor = fixture.run(&config, &["doctor", "--json"]);
    assert!(
        doctor.status.success(),
        "{}",
        String::from_utf8_lossy(&doctor.stderr)
    );
    let report: Value = serde_json::from_slice(&doctor.stdout).unwrap();
    assert_eq!(report["schema"], 1);
    assert_eq!(report["ok"], true);
    assert!(
        report["checks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|check| check["code"] == "watcher.status_missing")
    );
}

#[test]
fn watcher_recovers_when_a_provider_input_appears_and_reports_health() {
    let fixture = Fixture::new();
    let palette = fixture.path("palette.json");
    let output = fixture.path("foot.ini");
    let state = fixture.path("state.toml");
    let config = fixture.path("watch.toml");
    fs::write(
        &config,
        format!(
            r#"profiles_dir = "{}"
state_file = "{}"
default_profile = "glass"

[provider]
kind = "luau:noctalia"
[provider.inputs]
palette = "{}"

[targets.foot]
adapter = "luau:foot"
[targets.foot.outputs]
config = "{}"
[targets.foot.settings]
role = "terminal"
"#,
            fixture.path("profiles").display(),
            state.display(),
            palette.display(),
            output.display(),
        ),
    )
    .unwrap();

    let child = Command::new(env!("CARGO_BIN_EXE_theme-manager"))
        .env(
            "THEME_MANAGER_DATA_DIR",
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."),
        )
        .arg("--config")
        .arg(&config)
        .arg("watch")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let _guard = ChildGuard(child);
    thread::sleep(Duration::from_millis(200));
    fs::write(
        &palette,
        r##"{"primary":"#80d998","outline":"#7c9598","error":"#ffb4ab","shadow":"#000000","secondary":"#9ccaff","surface":"#0f1512","on_surface":"#dfe4de"}"##,
    )
    .unwrap();

    let deadline = Instant::now() + Duration::from_secs(5);
    let health_path = fixture.path("watch-status.json");
    let mut health = None;
    while Instant::now() < deadline {
        health = fs::read_to_string(&health_path)
            .ok()
            .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
            .filter(|value| value["last_success_ms"].is_number());
        if output.exists() && health.is_some() {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
    assert!(
        output.exists(),
        "watcher did not recover after provider input appeared"
    );
    let health = health.expect("watcher did not report a successful health update");
    assert_eq!(health["schema"], 1);
    assert!(health["last_success_ms"].is_number());
    assert!(health["last_error"].is_null());
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn components_does_not_require_a_home_or_config() {
    let output = Command::new(env!("CARGO_BIN_EXE_theme-manager"))
        .env_remove("HOME")
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("THEME_MANAGER_DATA_DIR")
        .args(["components", "--json"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let components: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(components["providers"].as_array().unwrap().len(), 0);
    assert_eq!(components["targets"].as_array().unwrap().len(), 0);
}

#[test]
fn official_noctalia_plugin_consumes_a_host_read_json_palette() {
    let fixture = Fixture::new();
    let palette = fixture.path("palette.json");
    let output = fixture.path("foot.ini");
    fs::write(
        &palette,
        r##"{"primary":"#80d998","outline":"#7c9598","error":"#ffb4ab","shadow":"#000000","secondary":"#9ccaff","surface":"#0f1512","on_surface":"#dfe4de"}"##,
    )
    .unwrap();
    let config = fixture.path("noctalia.toml");
    fs::write(
        &config,
        format!(
            r#"profiles_dir = "{}"
state_file = "{}"

[provider]
kind = "luau:noctalia"
[provider.inputs]
palette = "{}"

[targets.foot]
adapter = "luau:foot"
[targets.foot.outputs]
config = "{}"
[targets.foot.settings]
role = "terminal"
"#,
            fixture.path("profiles").display(),
            fixture.path("state.toml").display(),
            palette.display(),
            output.display(),
        ),
    )
    .unwrap();

    let apply = fixture.run(&config, &["apply", "glass"]);
    assert!(
        apply.status.success(),
        "{}",
        String::from_utf8_lossy(&apply.stderr)
    );
    assert!(fs::read_to_string(output).unwrap().contains("alpha=0.25"));
}

#[test]
fn dry_run_reports_unchanged_outputs_accurately() {
    let fixture = Fixture::new();
    let output = fixture.path("foot.ini");
    let config = fixture.write_config(&format!(
        r#"[targets.foot]
adapter = "luau:foot"
[targets.foot.outputs]
config = "{}"
[targets.foot.settings]
role = "terminal""#,
        output.display(),
    ));

    let first = fixture.run(&config, &["apply", "glass"]);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );

    let dry_run = fixture.run(&config, &["apply", "glass", "--dry-run", "--json"]);
    assert!(
        dry_run.status.success(),
        "{}",
        String::from_utf8_lossy(&dry_run.stderr)
    );
    let report: Value = serde_json::from_slice(&dry_run.stdout).unwrap();
    assert_eq!(report["targets"][0]["changed"].as_array().unwrap().len(), 0);
    assert_eq!(
        report["targets"][0]["unchanged"].as_array().unwrap().len(),
        1
    );
}

#[test]
fn render_failure_does_not_partially_write_earlier_targets() {
    let fixture = Fixture::new();
    let foot_output = fixture.path("foot.ini");
    let niri_output = fixture.path("niri.kdl");
    let config = fixture.write_config(&format!(
        r#"[targets.foot]
adapter = "luau:foot"
[targets.foot.outputs]
config = "{}"
[targets.foot.settings]
role = "terminal"

[targets.niri]
adapter = "luau:niri"
[targets.niri.outputs]
config = "{}"
[targets.niri.settings]

[[targets.niri.settings.windows]]
role = "missing"
app_id = "^broken$""#,
        foot_output.display(),
        niri_output.display(),
    ));

    let apply = fixture.run(&config, &["apply", "glass"]);
    assert!(!apply.status.success());
    assert!(!foot_output.exists());
}

#[test]
fn check_rejects_target_output_collisions() {
    let fixture = Fixture::new();
    let shared_output = fixture.path("shared.conf");
    let config = fixture.write_config(&format!(
        r#"[targets.foot]
adapter = "luau:foot"
[targets.foot.outputs]
config = "{}"
[targets.foot.settings]
role = "terminal"

[targets.niri]
adapter = "luau:niri"
[targets.niri.outputs]
config = "{}""#,
        shared_output.display(),
        shared_output.display(),
    ));

    let check = fixture.run(&config, &["check", "glass"]);
    assert!(!check.status.success());
    assert!(String::from_utf8_lossy(&check.stderr).contains("both write"));
}

#[test]
fn check_rejects_lexically_aliased_output_collisions() {
    let fixture = Fixture::new();
    let shared_output = fixture.path("shared.conf");
    let aliased_output = fixture.path("unused/../shared.conf");
    let config = fixture.write_config(&format!(
        r#"[targets.foot]
adapter = "luau:foot"
[targets.foot.outputs]
config = "{}"
[targets.foot.settings]
role = "terminal"

[targets.niri]
adapter = "luau:niri"
[targets.niri.outputs]
config = "{}""#,
        shared_output.display(),
        aliased_output.display(),
    ));

    let check = fixture.run(&config, &["check", "glass"]);
    assert!(!check.status.success());
    assert!(String::from_utf8_lossy(&check.stderr).contains("both write"));
}

#[test]
fn component_toggle_removes_and_restores_generated_output() {
    let fixture = Fixture::new();
    let output = fixture.path("foot.ini");
    let config = fixture.write_config(&format!(
        r#"[targets.foot]
adapter = "luau:foot"
[targets.foot.outputs]
config = "{}"
[targets.foot.settings]
role = "terminal""#,
        output.display(),
    ));

    let apply = fixture.run(&config, &["apply", "glass"]);
    assert!(
        apply.status.success(),
        "{}",
        String::from_utf8_lossy(&apply.stderr)
    );
    assert!(output.is_file());

    let disable = fixture.run(&config, &["components", "disable", "foot", "--json"]);
    assert!(
        disable.status.success(),
        "{}",
        String::from_utf8_lossy(&disable.stderr)
    );
    assert!(!output.exists());

    let apply_disabled = fixture.run(&config, &["apply", "glass"]);
    assert!(
        apply_disabled.status.success(),
        "{}",
        String::from_utf8_lossy(&apply_disabled.stderr)
    );
    assert!(!output.exists());

    let list = fixture.run(&config, &["components", "list", "--json"]);
    assert!(
        list.status.success(),
        "{}",
        String::from_utf8_lossy(&list.stderr)
    );
    let targets: Value = serde_json::from_slice(&list.stdout).unwrap();
    assert_eq!(targets[0]["target"], "foot");
    assert_eq!(targets[0]["enabled"], false);

    let enable = fixture.run(&config, &["components", "enable", "foot", "--json"]);
    assert!(
        enable.status.success(),
        "{}",
        String::from_utf8_lossy(&enable.stderr)
    );
    assert!(output.is_file());
}

#[test]
fn component_disable_refuses_to_delete_unmanaged_files() {
    let fixture = Fixture::new();
    let output = fixture.path("foot.ini");
    fs::write(&output, "# user-owned configuration\n").unwrap();
    let config = fixture.write_config(&format!(
        r#"[targets.foot]
adapter = "luau:foot"
[targets.foot.outputs]
config = "{}"
[targets.foot.settings]
role = "terminal""#,
        output.display(),
    ));

    let disable = fixture.run(&config, &["components", "disable", "foot"]);
    assert!(!disable.status.success());
    assert!(String::from_utf8_lossy(&disable.stderr).contains("not owned by theme-manager"));
    assert_eq!(
        fs::read_to_string(&output).unwrap(),
        "# user-owned configuration\n"
    );

    let list = fixture.run(&config, &["components", "list", "--json"]);
    let targets: Value = serde_json::from_slice(&list.stdout).unwrap();
    assert_eq!(targets[0]["enabled"], true);
}

#[test]
fn component_disable_refuses_modified_generated_files_even_with_marker() {
    let fixture = Fixture::new();
    let output = fixture.path("foot.ini");
    let config = fixture.write_config(&format!(
        r#"[targets.foot]
adapter = "luau:foot"
[targets.foot.outputs]
config = "{}"
[targets.foot.settings]
role = "terminal""#,
        output.display(),
    ));

    assert!(fixture.run(&config, &["apply", "glass"]).status.success());
    let mut modified = fs::read_to_string(&output).unwrap();
    modified.push_str("# manually changed\n");
    fs::write(&output, &modified).unwrap();

    let disable = fixture.run(&config, &["components", "disable", "foot"]);
    assert!(!disable.status.success());
    assert!(String::from_utf8_lossy(&disable.stderr).contains("has changed"));
    assert_eq!(fs::read_to_string(&output).unwrap(), modified);
}

#[test]
fn declaratively_disabled_component_cannot_be_enabled_at_runtime() {
    let fixture = Fixture::new();
    let output = fixture.path("foot.ini");
    let config = fixture.write_config(&format!(
        r#"[targets.foot]
enabled = false
adapter = "luau:foot"
[targets.foot.outputs]
config = "{}"
[targets.foot.settings]
role = "terminal""#,
        output.display(),
    ));

    let list = fixture.run(&config, &["components", "list", "--json"]);
    assert!(
        list.status.success(),
        "{}",
        String::from_utf8_lossy(&list.stderr)
    );
    let targets: Value = serde_json::from_slice(&list.stdout).unwrap();
    assert_eq!(targets[0]["configured"], true);
    assert_eq!(targets[0]["toggleable"], false);
    assert_eq!(targets[0]["enabled"], false);

    let enable = fixture.run(&config, &["components", "enable", "foot"]);
    assert!(!enable.status.success());
    assert!(String::from_utf8_lossy(&enable.stderr).contains("disabled in config"));
    assert!(!output.exists());
}

#[test]
fn failed_component_enable_rolls_back_runtime_state() {
    let fixture = Fixture::new();
    let output = fixture.path("foot.ini");
    let config = fixture.write_config(&format!(
        r#"[targets.foot]
adapter = "luau:foot"
[targets.foot.outputs]
config = "{}"
[targets.foot.settings]
role = "terminal""#,
        output.display(),
    ));

    assert!(fixture.run(&config, &["apply", "glass"]).status.success());
    assert!(
        fixture
            .run(&config, &["components", "disable", "foot"])
            .status
            .success()
    );
    assert!(!output.exists());

    fixture.write_config(&format!(
        r#"[targets.foot]
adapter = "luau:foot"
[targets.foot.outputs]
config = "{}"
[targets.foot.settings]
role = "missing""#,
        output.display(),
    ));
    let enable = fixture.run(&config, &["components", "enable", "foot"]);
    assert!(!enable.status.success());
    assert!(String::from_utf8_lossy(&enable.stderr).contains("rolled back"));
    assert!(!output.exists());

    let list = fixture.run(&config, &["components", "list", "--json"]);
    let targets: Value = serde_json::from_slice(&list.stdout).unwrap();
    assert_eq!(targets[0]["enabled"], false);
}

#[test]
fn generic_luau_target_is_discovered_applied_and_safely_toggled() {
    let fixture = Fixture::new();
    let plugin = fixture.path("plugins/generic-text");
    fs::create_dir_all(&plugin).unwrap();
    fs::write(
        plugin.join("plugin.toml"),
        r#"api = 1
id = "generic-text"
kind = "target"
entry = "main.luau"
"#,
    )
    .unwrap();
    fs::write(
        plugin.join("main.luau"),
        r#"return {
    render = function(ctx)
        return { document = "{\"profile\":\"" .. ctx.theme.profile .. "\"}\n" }
    end,
}
"#,
    )
    .unwrap();

    let output = fixture.path("application/theme.json");
    let config = fixture.write_config(&format!(
        r#"[extensions]
dirs = ["{}"]

[targets.desktop-theme]
adapter = "luau:generic-text"

[targets.desktop-theme.outputs]
document = "{}""#,
        fixture.path("plugins").display(),
        output.display(),
    ));

    let plugins = fixture.run(&config, &["plugins", "check"]);
    assert!(
        plugins.status.success(),
        "{}",
        String::from_utf8_lossy(&plugins.stderr)
    );

    let apply = fixture.run(&config, &["apply", "glass"]);
    assert!(
        apply.status.success(),
        "{}",
        String::from_utf8_lossy(&apply.stderr)
    );
    assert_eq!(
        fs::read_to_string(&output).unwrap(),
        "{\"profile\":\"glass\"}\n"
    );

    let list = fixture.run(&config, &["components", "list", "--json"]);
    let targets: Value = serde_json::from_slice(&list.stdout).unwrap();
    let dynamic = targets
        .as_array()
        .unwrap()
        .iter()
        .find(|target| target["target"] == "desktop-theme")
        .unwrap();
    assert_eq!(dynamic["adapter"], "luau:generic-text");

    let disable = fixture.run(&config, &["components", "disable", "desktop-theme"]);
    assert!(
        disable.status.success(),
        "{}",
        String::from_utf8_lossy(&disable.stderr)
    );
    assert!(!output.exists());

    assert!(
        fixture
            .run(&config, &["components", "enable", "desktop-theme"])
            .status
            .success()
    );
    fs::write(&output, "user changed this file\n").unwrap();
    let refuse = fixture.run(&config, &["components", "disable", "desktop-theme"]);
    assert!(!refuse.status.success());
    assert!(String::from_utf8_lossy(&refuse.stderr).contains("has changed"));
    assert!(output.exists());
}
