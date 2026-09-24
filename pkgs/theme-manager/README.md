# theme-manager v0.1

A provider/target-based desktop appearance engine with CLI and TUI frontends.

## Build and run

The Nix package is self-contained. It includes the integration-free Rust engine,
official Static, Noctalia, Niri, and Foot Luau plugins, the `glass` and `paper`
profiles, and the optional Noctalia bridge assets.
No files need to be copied into `~/.config` before first use:

```bash
nix build path:.
./result/bin/theme-manager check
./result/bin/theme-manager apply glass
./result/bin/theme-manager status
```

You can also run it directly with `nix run path:. -- status`.

If `$XDG_CONFIG_HOME/theme-manager/config.toml` exists, it takes precedence over
the packaged configuration. This keeps the zero-configuration default useful
while allowing the provider, targets, output paths, and profile directory to be
customized normally.

For Home Manager, add this flake as an input and enable its module:

```nix
{
  imports = [ inputs.theme-manager.homeManagerModules.default ];
  programs.theme-manager.enable = true;
}
```

The module installs the package and starts the watcher. When Home Manager has
Niri or Foot enabled, it also adds the generated fragment to that application's
configuration. The packaged defaults therefore work without checkout paths or
manual config symlinks.

This is the first extraction of the old Noctalia-specific Theme Manager into an
independent Rust application. Noctalia is now optional and only provides
dynamic colors; it does not own the engine or its user interface.

## Design

```text
structure profile (source of truth)
              +
      dynamic color provider
              |
              v
        resolved theme
              |
       target adapters
       /           \
    Niri           Foot

optional frontends:
  CLI
  terminal UI
```

The boundaries are deliberate:

- **Profile**: appearance intent only (`glass`, `paper`, semantic roles).
- **Provider plugin**: turns declared inputs/settings into semantic colors.
- **Resolved theme**: canonical, implementation-independent data.
- **Target plugin**: translates the resolved theme into named text artifacts.
- **CLI/runtime**: public API, persistent state, and orchestration.
- **Terminal UI**: beginner-friendly control surface over the same runtime API.

## Reused libraries

The engine intentionally uses mature crates instead of hand-rolling infrastructure:

- `clap` — CLI parsing
- `serde` + `toml` + `serde_json` — config/profile/data formats
- `notify` — filesystem watching / hot reload
- `ratatui` + `crossterm` — interactive terminal interface
- `anyhow` — contextual application errors
- `tracing` + `tracing-subscriber` — runtime logging

Rust remains the trusted engine. Plugins only transform in-memory data; they
cannot read or write arbitrary files, launch processes, or access the network.

## Workspace

```text
crates/
  theme-core/                 schemas, resolver, engine, atomic output
  theme-extension-luau/       sandbox, plugin discovery, versioned host API
  theme-manager/              CLI/TUI, empty registry, live watcher

plugins/
  static/                     fixed semantic palette provider
  noctalia/                   rendered JSON palette provider
  niri/                       Niri KDL target
  foot/                       Foot background-alpha target

integrations/
  noctalia/
    palette.template.json     dynamic color bridge

examples/
  config.toml
  profiles/
    glass.toml
    paper.toml
```

The Rust crates contain **no registered application integrations**. A bare
binary discovers zero adapters. Integration names and rendering logic live in
independent plugin directories and are found at runtime.

## Structure profiles

Profiles use semantic roles rather than application names:

```toml
[roles.browser]
background_opacity = "secondary"
blur = true
corner_radius = "primary"
accent_color = "primary"

[roles.terminal]
background_opacity = "primary"
blur = true
corner_radius = "primary"
```

The machine config decides how those roles map to concrete software:

```toml
[targets.niri]
adapter = "luau:niri"

[targets.niri.outputs]
config = "$XDG_CONFIG_HOME/theme-manager/generated/niri.kdl"

[targets.niri.settings]
global_role = "compositor"

[[targets.niri.settings.windows]]
role = "browser"
app_id = "^firefox$"
use_opacity = true

[[targets.niri.settings.windows]]
role = "terminal"
app_id = "^(foot|footclient)$"
use_opacity = false

[targets.foot]
adapter = "luau:foot"

[targets.foot.outputs]
config = "$XDG_CONFIG_HOME/theme-manager/generated/foot.ini"

[targets.foot.settings]
role = "terminal"
```

A future integration can consume any semantic role without changing
`glass.toml` or `paper.toml`; the engine has no terminal-specific category.

## Dynamic providers

The profile refers to semantic colors such as:

```toml
active_color = "primary"
inactive_color = "outline"
```

It does not know where those colors originate.

### Static

```toml
[provider]
kind = "luau:static"

[provider.settings.colors]
primary = "#80d998"
outline = "#7c9598"
error = "#ffb4ab"
shadow = "#000000"
secondary = "#9ccaff"
```

### Noctalia

```toml
[provider]
kind = "luau:noctalia"

[provider.inputs]
palette = "$XDG_CACHE_HOME/theme-manager/noctalia-palette.json"
```

See `integrations/noctalia/README.md`. Noctalia renders a tiny JSON template
when its palette changes. The Rust watcher sees that file change and reapplies
the current structure profile.

## CLI

```bash
theme-manager list
theme-manager status
theme-manager apply glass
theme-manager apply paper
theme-manager resolve glass
theme-manager resolve glass --json
theme-manager check
theme-manager components
theme-manager components list
theme-manager components disable foot
theme-manager components enable foot
theme-manager plugins list
theme-manager plugins check
theme-manager tui
theme-manager tui --no-vim
theme-manager watch
```

`components` without a subcommand shows discovered plugin adapters.
`components list` shows configured target instances and whether each is
disabled declaratively, disabled at runtime, or enabled.

Runtime target switches are stored in the state file. Disabling a target removes
its generated fragment so an optional Niri/Foot include stops applying it.
Enabling a target regenerates its fragment immediately. A target with
`enabled = false` in `config.toml` remains a hard declarative disable and cannot
be overridden at runtime.

## Luau extensions

Theme-Manager supports provider and target plugins from the packaged
`$THEME_MANAGER_DATA_DIR/plugins` directory and the user-owned
`$XDG_CONFIG_HOME/theme-manager/plugins` directory. Each immediate child has a
`plugin.toml` manifest and a `.luau` entry script. Additional search directories
can be configured without removing either default:

```toml
[extensions]
dirs = ["$XDG_CONFIG_HOME/theme-manager/extra-plugins"]

[extensions.limits]
memory_mib = 32
execution_ms = 250
io_mib = 4
```

Target scripts receive the fully resolved theme and return named strings. Rust
maps those names to configured paths and performs all writes:

```toml
[targets.my-output]
adapter = "luau:generic-text"

[targets.my-output.outputs]
document = "$XDG_CONFIG_HOME/my-application/theme.conf"

[targets.my-output.settings]
role = "compositor"
```

Provider scripts receive only explicitly declared input file contents. A small
host-provided JSON codec is available for structured inputs. Scripts
cannot access arbitrary files, processes, environment variables, or the
network. Relative `require("./module")` imports are confined to the plugin
directory. Use `theme-manager plugins check` to compile every discovered
extension. The packaged `share/theme-manager/luau/theme-manager.d.luau` file
documents API v1 for editor tooling, and `share/theme-manager/examples/plugins`
contains a generic target example.

The JSON forms are intended for frontends/scripts:

```bash
theme-manager list --json
theme-manager status --json
theme-manager apply glass --json
theme-manager components list --json
theme-manager components enable foot --json
```

## Terminal interface

Run `theme-manager tui` for an interactive interface that does not require
memorizing CLI commands. It supports:

- browsing and applying profiles
- dry-run previews
- validating every profile and target
- enabling/disabling configured targets
- live config/state reloads
- responsive wide and narrow terminal layouts

The controls are always shown at the bottom. Vim navigation is enabled by
default: `h`/`l` select panes, `j`/`k` move, `g`/`G` jump to the
first/last item, and `Ctrl-u`/`Ctrl-d` jump half a list. Arrow keys, `Tab`,
`Home`, and `End` work in either mode. `Enter` applies or toggles, `d`
previews, `c` validates, `r` reloads, and `q` exits.

Disable Vim keys persistently in `config.toml`:

```toml
[tui]
vim_keys = false
```

Or disable them for one invocation with `theme-manager tui --no-vim`.

## Hot swapping

`theme-manager watch` watches:

- the profile directory
- the active-state directory
- the runtime config directory
- provider-owned watch paths (for example Noctalia's rendered palette JSON)

After each event it rebuilds the runtime registry from `config.toml` and applies
the active profile. This means you can change profile values, switch the active
profile, change provider settings, enable/disable targets, or alter target
bindings without restarting the watcher. Config and profile symlinks are
resolved to their source directories, so editing Home Manager
`mkOutOfStoreSymlink` sources also triggers a reload.

Configuration and profile structures reject unknown fields. A misspelled
option therefore fails `theme-manager check` instead of being silently ignored.

All providers and targets use the same versioned Luau boundary, including the
official plugins. The registry begins empty, so adding a future application is
a plugin/package change rather than a Rust rebuild. Rust still owns discovery,
path expansion, bounded reads, schema and palette validation, collision checks,
render transactions, atomic writes, ownership hashes, state, and watching.

## Development

`nix develop` provides Cargo, rustc, rust-analyzer, clippy, and rustfmt. A source
build can use the example dynamic configuration explicitly:

```bash
cargo run -p theme-manager -- --config examples/config.toml components
```

## Niri / Foot integration

Do not let the runtime overwrite Home Manager's immutable generated files.
Instead include the mutable generated fragments.

Niri:

```kdl
include optional=true "~/.config/theme-manager/generated/niri.kdl"
```

Foot:

```ini
include=~/.config/theme-manager/generated/foot.ini
```

Niri live-reloads its include. Foot reads the generated alpha for newly created
terminal instances; v0.1 does not inject control sequences into existing PTYs.

## Current scope

v0.1 intentionally focuses on the stable path we already proved:

- semantic structure profiles
- dynamic/static color provider plugins
- Niri layout, focus ring, shadow, window radius/blur/opacity plugin
- Foot native background alpha plugin
- CLI, JSON API, and interactive terminal UI
- live watcher
- optional Noctalia color provider

Firefox currently receives compositor-level structure through the `browser`
role bound by the Niri adapter. A future Firefox target can own browser-internal
CSS/accent behavior without changing the structure profile.
