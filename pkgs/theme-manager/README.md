# Theme Manager

Theme Manager applies portable Structure Profiles and named Color Palettes as a
validated Theme Bundle. Application output is produced by fixed built-in wires;
there is no runtime plugin system or arbitrary command provider.

## Model

- A **Color Provider** produces a normalized semantic palette. Built-ins are
  `static`, `json-file`, and `noctalia`.
- A **Color Palette** names one provider input.
- A **Structure Profile** contains portable opacity, blur, radius, gap,
  focus-ring, and shadow values.
- A **Theme Bundle** selects one palette and one structure. Optional cursor,
  typography, icon, and wallpaper selections must name both an Appearance
  Library and an item.
- An **Application Wire** selects a fixed Niri, Foot, or constrained Noctalia
  adapter.
- An **Application Contract** lists the small set of required and optional
  capabilities expected for an application.
- The **Capability Plan** combines trusted adapter claims with the read-only Nix
  inventory and rejects missing or duplicate ownership before writing files.

`check`, `status`, and `apply` use the same Capability Plan. Claims include the
owner, source, concrete setting boundary, and `runtime`, `session`, or `rebuild`
cadence.

## CLI

```text
theme-manager theme list
theme-manager theme show glass
theme-manager structure list
theme-manager palette list
theme-manager library list stylix-cursors
theme-manager status
theme-manager check [theme]
theme-manager apply <theme> [--dry-run]
theme-manager watch
```

Add `--json` to inspection, check, status, and apply commands where offered.

## Configuration

The packaged default is a complete static example. The essential shape is:

```toml
profiles_dir = "$XDG_CONFIG_HOME/theme-manager/profiles"
state_file = "$XDG_STATE_HOME/theme-manager/state.toml"
generated_dir = "$XDG_CONFIG_HOME/theme-manager/generated"
default_theme = "glass"

[color_providers.static]
kind = "static"

[color_palettes.fallback]
provider = "static"
source = "static"

[color_palettes.fallback.colors]
primary = "#80d998"
outline = "#7c9598"
error = "#ffb4ab"
shadow = "#000000"

[theme_bundles.glass]
structure = "glass"
palette = "fallback"

[application_contracts.foot]
required = ["background_alpha"]

[application_wires.foot-alpha]
adapter = "foot"
role = "terminal"
```

All typed boundaries reject unknown fields. Built-in adapters own fixed paths
beneath `generated_dir`: `niri.kdl` and `foot.ini`. Headless Noctalia templates
may select only contained relative outputs; absolute paths, `..`, and symlink
escapes are rejected.

## Noctalia

Noctalia has two independent roles:

1. A Color Provider reads a shell-produced normalized JSON palette or invokes
   `noctalia theme` headlessly from an image/theme JSON.
2. A `noctalia-template` wire either renders a repository-owned template in a
   private staging directory or delegates an audited template ID to a running
   shell.

Headless template IDs are closed to `repository:palette`,
`repository:foot-colors`, and `repository:pywalfox`; their reviewed template
bytes are embedded in the binary. Configuration supplies no template path,
hook, dynamic output, or command.

Headless palettes support `m3-tonal-spot` (the explicit default), `m3-content`,
`m3-fruit-salad`, `m3-rainbow`, `m3-monochrome`, `vibrant`, `faithful`, `soft`,
`dysfunctional`, and `muted`. The CLI executable is fixed; configuration cannot
provide a command or hook.

The repository-owned Foot and Pywalfox templates live under
`integrations/noctalia`. No community template is fetched at runtime.
The Home Manager Noctalia module reads the same Theme Manager TOML and derives
its built-in/user template selection from enabled shell-driven wires; template
IDs are not maintained in a second selection list.

## Appearance libraries and Stylix

Stylix is optional and is not a Theme Manager dependency. When used, Nix owns
the package-valued library definitions and emits a read-only inventory. A bundle
must use a qualified reference such as:

```toml
cursor = { library = "stylix-cursors", item = "bibata-modern-ice" }
```

Theme Manager rejects unknown libraries/items, mismatched library kinds,
uninstalled or ineffective rebuild selections, and missing required consumer
coverage. It never runs a Nix rebuild.

The repository hosts currently select
`stylix-cursors/bibata-modern-ice`. The local Nix module reads that qualified
selection from the same runtime bundle configuration, configures the pinned
Bibata package as `Bibata-Modern-Ice` at size 24 through upstream Stylix, enables
only its GTK and X11 cursor subtargets, and generates the read-only inventory.
No Stylix color or application target is enabled.

## Safety and update behavior

- Every provider result and profile is validated before the first live write.
- Every Theme Manager output stays beneath one exclusive generated directory.
- Outputs left by removed or disabled wires are deleted from that exclusive
  directory after the replacement set has rendered successfully.
- Files are atomically replaced and unchanged contents are not rewritten.
- A provider/render failure preserves all last-valid files.
- Shell-driven Noctalia output is a separate delegated transaction and is
  reported as such.
- The watcher observes parent directories, so atomic file replacement remains
  visible, and debounces event bursts.
- Persistent state contains only the active Theme Bundle. v0.2
  `active_profile` state is migrated on the next successful apply.

## Development

```text
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
nix flake check
```

The Phase 0 evidence is in `PHASE-0-INVENTORY.md`; the completed design and its
decisions remain in `PLAN.md`.
