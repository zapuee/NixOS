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
        for name in ["glass", "paper"] {
            fs::write(
                profiles.join(format!("{name}.toml")),
                match name {
                    "glass" => include_str!("../../../profiles/glass.toml"),
                    "paper" => include_str!("../../../profiles/paper.toml"),
                    _ => unreachable!(),
                },
            )
            .unwrap();
        }
        Self { root }
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    fn write_config(&self, body: &str) -> PathBuf {
        let path = self.path("config.toml");
        fs::write(
            &path,
            format!(
                r#"profiles_dir = "{}"
state_file = "{}"
generated_dir = "{}"
default_theme = "glass"

{body}
"#,
                self.path("profiles").display(),
                self.path("state.toml").display(),
                self.path("generated").display(),
            ),
        )
        .unwrap();
        path
    }

    fn standard_config(&self) -> PathBuf {
        self.write_config(STANDARD_BODY)
    }

    fn command(&self, config: &Path) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_theme-manager"));
        command.arg("--config").arg(config);
        command
    }

    fn run(&self, config: &Path, args: &[&str]) -> Output {
        self.command(config).args(args).output().unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

const STANDARD_BODY: &str = r##"
[color_providers.static]
kind = "static"

[color_palettes.fallback]
provider = "static"
source = "static"
[color_palettes.fallback.colors]
background = "#0f1512"
on_background = "#dfe4de"
surface = "#0f1512"
on_surface = "#dfe4de"
primary = "#80d998"
on_primary = "#00210b"
secondary = "#9ccaff"
on_secondary = "#001d33"
error = "#ffb4ab"
on_error = "#690005"
outline = "#7c9598"
shadow = "#000000"

[theme_bundles.glass]
structure = "glass"
palette = "fallback"
[theme_bundles.paper]
structure = "paper"
palette = "fallback"

[application_contracts.niri]
required = ["gaps", "focus_ring", "shadow", "corner_radius"]
[application_contracts.foot]
required = ["background_alpha"]
optional = ["window_structure"]

[application_wires.niri]
adapter = "niri"
global_role = "compositor"
[[application_wires.niri.windows]]
application = "firefox"
role = "browser"
app_id = "^firefox$"
opacity_owner = "compositor"
[[application_wires.niri.windows]]
application = "foot"
role = "terminal"
app_id = "^(foot|footclient)$"
opacity_owner = "application"

[application_wires.foot-alpha]
adapter = "foot"
role = "terminal"
"##;

#[test]
fn glass_and_paper_match_v0_2_golden_outputs() {
    let fixture = Fixture::new();
    let config = fixture.standard_config();
    for (theme, expected_foot, expected_niri) in [
        (
            "glass",
            include_str!("fixtures/v0.2/glass/foot.ini"),
            include_str!("fixtures/v0.2/glass/niri.kdl"),
        ),
        (
            "paper",
            include_str!("fixtures/v0.2/paper/foot.ini"),
            include_str!("fixtures/v0.2/paper/niri.kdl"),
        ),
    ] {
        let apply = fixture.run(&config, &["apply", theme, "--json"]);
        assert!(
            apply.status.success(),
            "{}",
            String::from_utf8_lossy(&apply.stderr)
        );
        assert_eq!(
            fs::read_to_string(fixture.path("generated/foot.ini")).unwrap(),
            expected_foot
        );
        assert_eq!(
            fs::read_to_string(fixture.path("generated/niri.kdl")).unwrap(),
            format!("{expected_niri}\n")
        );
        let report: Value = serde_json::from_slice(&apply.stdout).unwrap();
        assert_eq!(report["theme"], theme);
        assert_eq!(report["capability_plan"]["complete"], true);
    }
}

#[test]
fn dry_run_renders_and_validates_without_writing() {
    let fixture = Fixture::new();
    let config = fixture.standard_config();
    let output = fixture.run(&config, &["apply", "glass", "--dry-run"]);
    assert!(output.status.success());
    assert!(!fixture.path("generated").exists());
    assert!(!fixture.path("state.toml").exists());
}

#[test]
fn missing_required_capability_fails_before_output_changes() {
    let fixture = Fixture::new();
    let config = fixture.write_config(&STANDARD_BODY.replace(
        "required = [\"background_alpha\"]",
        "required = [\"background_alpha\", \"colors\"]",
    ));
    let output = fixture.run(&config, &["apply", "glass"]);
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("missing required capability 'colors'")
    );
    assert!(!fixture.path("generated").exists());
}

#[test]
fn duplicate_capability_owners_are_rejected() {
    let fixture = Fixture::new();
    let config = fixture.write_config(&format!(
        "{STANDARD_BODY}\n[application_wires.second-foot]\nadapter = \"foot\"\nrole = \"terminal\""
    ));
    let output = fixture.run(&config, &["check", "glass"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("multiple owners"));
}

#[test]
fn invalid_palette_preserves_last_valid_outputs() {
    let fixture = Fixture::new();
    let config = fixture.standard_config();
    assert!(fixture.run(&config, &["apply", "glass"]).status.success());
    let niri = fixture.path("generated/niri.kdl");
    let before = fs::read_to_string(&niri).unwrap();
    fs::write(
        &config,
        fs::read_to_string(&config)
            .unwrap()
            .replace("#80d998", "not-a-color"),
    )
    .unwrap();
    let failed = fixture.run(&config, &["apply", "paper"]);
    assert!(!failed.status.success());
    assert_eq!(fs::read_to_string(niri).unwrap(), before);
}

#[test]
fn runtime_works_without_a_niri_wire() {
    let fixture = Fixture::new();
    let body = STANDARD_BODY
        .replace(
            "[application_contracts.niri]\nrequired = [\"gaps\", \"focus_ring\", \"shadow\", \"corner_radius\"]\n",
            "",
        )
        .split("[application_wires.niri]")
        .next()
        .unwrap()
        .to_string()
        + "\n[application_wires.foot-alpha]\nadapter = \"foot\"\nrole = \"terminal\"\n";
    let config = fixture.write_config(&body);
    let output = fixture.run(&config, &["apply", "glass"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(fixture.path("generated/foot.ini").exists());
    assert!(!fixture.path("generated/niri.kdl").exists());
}

#[test]
fn removed_wires_leave_no_stale_generated_output() {
    let fixture = Fixture::new();
    let config = fixture.standard_config();
    assert!(fixture.run(&config, &["apply", "glass"]).status.success());
    assert!(fixture.path("generated/niri.kdl").exists());

    let foot_only = STANDARD_BODY
        .replace(
            "[application_contracts.niri]\nrequired = [\"gaps\", \"focus_ring\", \"shadow\", \"corner_radius\"]\n",
            "",
        )
        .split("[application_wires.niri]")
        .next()
        .unwrap()
        .to_string()
        + "\n[application_wires.foot-alpha]\nadapter = \"foot\"\nrole = \"terminal\"\n";
    fixture.write_config(&foot_only);
    let output = fixture.run(&config, &["apply", "glass", "--json"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!fixture.path("generated/niri.kdl").exists());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(report["wires"].as_array().unwrap().iter().any(|wire| {
        wire["wire"] == "cleanup"
            && wire["changed"][0]
                .as_str()
                .unwrap()
                .ends_with("generated/niri.kdl")
    }));
}

#[test]
fn a_wire_can_override_the_bundle_palette() {
    let fixture = Fixture::new();
    let body = STANDARD_BODY
        .replace(
            "[theme_bundles.glass]",
            "[color_palettes.alternate]\nprovider = \"static\"\nsource = \"static\"\n[color_palettes.alternate.colors]\nprimary = \"#ff0000\"\noutline = \"#00ff00\"\nerror = \"#0000ff\"\nshadow = \"#ffffff\"\n\n[theme_bundles.glass]",
        )
        .replace(
            "adapter = \"niri\"\nglobal_role",
            "adapter = \"niri\"\npalette = \"alternate\"\nglobal_role",
        );
    let config = fixture.write_config(&body);
    let output = fixture.run(&config, &["apply", "glass"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let niri = fs::read_to_string(fixture.path("generated/niri.kdl")).unwrap();
    assert!(niri.contains("rgba(255, 0, 0, 0.4)"));
}

#[cfg(unix)]
#[test]
fn shared_palette_is_loaded_once_per_apply() {
    use std::os::unix::fs::PermissionsExt;
    let fixture = Fixture::new();
    let fake = fixture.path("noctalia-counting");
    let counter = fixture.path("count");
    fs::write(
        &fake,
        r##"#!/bin/sh
count=0
if [ -f "$THEME_MANAGER_COUNT_FILE" ]; then read -r count < "$THEME_MANAGER_COUNT_FILE"; fi
count=$((count + 1))
printf '%s\n' "$count" > "$THEME_MANAGER_COUNT_FILE"
printf '{"primary":"#80d998","outline":"#7c9598","error":"#ffb4ab","shadow":"#000000"}\n'
"##,
    )
    .unwrap();
    fs::set_permissions(&fake, fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(fixture.path("image.png"), "fake").unwrap();
    let body = STANDARD_BODY
        .replace(
            "[color_providers.static]\nkind = \"static\"",
            "[color_providers.static]\nkind = \"noctalia\"",
        )
        .replace(
            "provider = \"static\"\nsource = \"static\"\n[color_palettes.fallback.colors]",
            &format!(
                "provider = \"static\"\nsource = \"image\"\nimage = \"{}\"\n[unused]",
                fixture.path("image.png").display()
            ),
        );
    let body = body.split("[unused]").next().unwrap().to_string()
        + &STANDARD_BODY[STANDARD_BODY.find("[theme_bundles.glass]").unwrap()..];
    let config = fixture.write_config(&body);
    let output = fixture
        .command(&config)
        .env("THEME_MANAGER_NOCTALIA", &fake)
        .env("THEME_MANAGER_COUNT_FILE", &counter)
        .args(["apply", "glass"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read_to_string(counter).unwrap().trim(), "1");
}

#[test]
fn watcher_recovers_when_json_palette_appears() {
    let fixture = Fixture::new();
    let palette = fixture.path("palette.json");
    let body = STANDARD_BODY
        .replace(
            "[color_providers.static]\nkind = \"static\"",
            "[color_providers.static]\nkind = \"json-file\"",
        )
        .replace(
            "provider = \"static\"\nsource = \"static\"\n[color_palettes.fallback.colors]",
            &format!(
                "provider = \"static\"\nsource = \"json-file\"\npath = \"{}\"\n[unused]",
                palette.display()
            ),
        );
    let body = body.split("[unused]").next().unwrap().to_string()
        + &STANDARD_BODY[STANDARD_BODY.find("[theme_bundles.glass]").unwrap()..];
    let config = fixture.write_config(&body);
    let child = fixture
        .command(&config)
        .arg("watch")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let _guard = ChildGuard(child);
    thread::sleep(Duration::from_millis(200));
    fs::write(
        &palette,
        r##"{"primary":"#80d998","outline":"#7c9598","error":"#ffb4ab","shadow":"#000000"}"##,
    )
    .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline && !fixture.path("generated/foot.ini").exists() {
        thread::sleep(Duration::from_millis(50));
    }
    assert!(fixture.path("generated/foot.ini").exists());
}

#[cfg(unix)]
#[test]
fn watcher_reapplies_an_image_palette_without_changing_its_scheme() {
    use std::os::unix::fs::PermissionsExt;
    let fixture = Fixture::new();
    let fake = fixture.path("noctalia-image");
    fs::write(
        &fake,
        r##"#!/bin/sh
image="$2"
shift 2
scheme=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --scheme) scheme="$2"; shift 2 ;;
    *) shift ;;
  esac
done
[ "$scheme" = "muted" ] || exit 2
if [ "$(sed -n '1p' "$image")" = "red" ]; then
  primary="#aa0000"
else
  primary="#2f6846"
fi
printf '{"primary":"%s","outline":"#7c9598","error":"#ffb4ab","shadow":"#000000"}\n' "$primary"
"##,
    )
    .unwrap();
    fs::set_permissions(&fake, fs::Permissions::from_mode(0o700)).unwrap();
    let image = fixture.path("wallpaper.png");
    fs::write(&image, "green\n").unwrap();
    let config = fixture.write_config(&format!(
        r#"[color_providers.noctalia]
kind = "noctalia"
[color_palettes.wallpaper]
provider = "noctalia"
source = "image"
image = "{}"
scheme = "muted"
[theme_bundles.glass]
structure = "glass"
palette = "wallpaper"
[application_contracts.niri]
required = ["gaps", "focus_ring", "shadow", "corner_radius"]
[application_wires.niri]
adapter = "niri"
global_role = "compositor""#,
        image.display()
    ));
    let child = fixture
        .command(&config)
        .env("THEME_MANAGER_NOCTALIA", &fake)
        .arg("watch")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let _guard = ChildGuard(child);
    let niri = fixture.path("generated/niri.kdl");
    let initial_deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < initial_deadline
        && !fs::read_to_string(&niri)
            .is_ok_and(|contents| contents.contains("rgba(47, 104, 70, 0.4)"))
    {
        thread::sleep(Duration::from_millis(50));
    }
    assert!(
        fs::read_to_string(&niri)
            .unwrap()
            .contains("rgba(47, 104, 70, 0.4)")
    );

    fs::write(&image, "red\n").unwrap();
    let changed_deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < changed_deadline
        && !fs::read_to_string(&niri)
            .is_ok_and(|contents| contents.contains("rgba(170, 0, 0, 0.4)"))
    {
        thread::sleep(Duration::from_millis(50));
    }
    assert!(
        fs::read_to_string(niri)
            .unwrap()
            .contains("rgba(170, 0, 0, 0.4)")
    );
}

#[cfg(unix)]
#[test]
fn headless_noctalia_template_is_staged_and_contained() {
    use std::os::unix::fs::PermissionsExt;
    let fixture = Fixture::new();
    let fake = fixture.path("noctalia");
    fs::write(
        &fake,
        r##"#!/bin/sh
spec=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    -r|--render) spec="$2"; shift 2 ;;
    *) shift ;;
  esac
done
if [ -n "$spec" ]; then
  output="${spec#*:}"
  printf 'rendered\n' > "$output"
else
  printf '{"primary":"#80d998","outline":"#7c9598","error":"#ffb4ab","shadow":"#000000"}\n'
fi
"##,
    )
    .unwrap();
    fs::set_permissions(&fake, fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(fixture.path("theme.json"), "{}").unwrap();
    let config = fixture.write_config(&format!(
        r#"[color_providers.noctalia]
kind = "noctalia"
[color_palettes.native]
provider = "noctalia"
source = "theme-json"
theme_json = "{}"
[theme_bundles.glass]
structure = "glass"
palette = "native"
[application_wires.example]
adapter = "noctalia-template"
palette = "native"
execution = "headless"
template = "repository:palette"
output = "noctalia/example.txt""#,
        fixture.path("theme.json").display(),
    ));
    let output = fixture
        .command(&config)
        .env("THEME_MANAGER_NOCTALIA", &fake)
        .args(["apply", "glass"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(fixture.path("generated/noctalia/example.txt")).unwrap(),
        "rendered\n"
    );
}

#[test]
fn escaping_headless_output_is_rejected_by_typed_config() {
    let fixture = Fixture::new();
    let config = fixture.write_config(
        r#"[color_providers.noctalia]
kind = "noctalia"
[color_palettes.native]
provider = "noctalia"
source = "theme-json"
theme_json = "/tmp/theme.json"
[theme_bundles.glass]
structure = "glass"
palette = "native"
[application_wires.example]
adapter = "noctalia-template"
palette = "native"
execution = "headless"
template = "repository:palette"
output = "../escape""#,
    );
    let output = fixture.run(&config, &["check", "glass"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("beneath generated_dir"));
}

#[test]
fn noctalia_template_wires_require_matching_native_palette_authority() {
    let fixture = Fixture::new();
    let static_config = fixture.write_config(&format!(
        "{STANDARD_BODY}\n[application_wires.template]\nadapter = \"noctalia-template\"\npalette = \"fallback\"\nexecution = \"shell\"\ntemplate = \"builtin:starship\""
    ));
    let output = fixture.run(&static_config, &["check", "glass"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Noctalia-backed palette"));

    let image_config = fixture.write_config(&format!(
        r#"[color_providers.noctalia]
kind = "noctalia"
[color_palettes.native]
provider = "noctalia"
source = "image"
image = "{}"
[theme_bundles.glass]
structure = "glass"
palette = "native"
[application_wires.template]
adapter = "noctalia-template"
palette = "native"
execution = "shell"
template = "builtin:starship""#,
        fixture.path("wallpaper.png").display()
    ));
    let output = fixture.run(&image_config, &["check", "glass"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("shell-json palette authority"));
}
