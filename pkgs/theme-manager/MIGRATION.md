# Migration from the Noctalia-hosted prototype

Do not replace the working setup immediately. Run v0.1 beside it, verify the
generated fragments, then remove the old Theme Manager service.

## Recommended repo placement

The Rust engine should no longer live under the Noctalia desktop-shell module.
A cleaner repository split is:

```text
modules/user/programs/theme-manager/
  <this Rust workspace or its Nix wrapper>

modules/user/environments/desktop-shells/noctalia/
  default.nix
  rices/
  integration/theme-manager.nix   # optional provider wiring only
```

That keeps `theme-manager` installable when Noctalia is not selected.

## 1. Install the self-contained package

The Nix package carries its default config and profiles. Install the package or
enable the exported Home Manager module; no config symlinks are needed. Create
`$XDG_CONFIG_HOME/theme-manager/config.toml` only when overriding the packaged
static provider or target settings.

## Plugin-only configuration

Providers and targets are no longer compiled adapters. Existing custom configs
must use the namespaced plugin IDs and separate host-owned paths from
plugin-owned settings:

```toml
[provider]
kind = "luau:static"

[provider.settings.colors]
primary = "#80d998"

[targets.foot]
adapter = "luau:foot"

[targets.foot.outputs]
config = "$XDG_CONFIG_HOME/theme-manager/generated/foot.ini"

[targets.foot.settings]
role = "terminal"
```

Noctalia uses `kind = "luau:noctalia"` and declares its palette file under
`[provider.inputs]`. Niri uses `adapter = "luau:niri"`, a named `config`
output, and places `global_role` plus `windows` under
`[targets.niri.settings]`. The packaged default config already uses this form.

## 2. Niri

Keep this in your declarative Niri config:

```kdl
include optional=true "~/.config/theme-manager/generated/niri.kdl"
```

The new Rust target owns that one generated fragment. It never edits the main
Home Manager-managed `config.kdl`.

## 3. Foot

Include:

```ini
include=~/.config/theme-manager/generated/foot.ini
```

after the palette/theme include so structure alpha wins.

## 4. Noctalia dynamic colors

Register `integrations/noctalia/palette.template.json` as a Noctalia user
template and output it to:

```text
$XDG_CACHE_HOME/theme-manager/noctalia-palette.json
```

This replaces the old design where Noctalia rendered Niri's KDL itself. Now it
only exports semantic palette data; Rust owns all target generation.

## 5. Verify before enabling watch mode

```bash
theme-manager check
theme-manager resolve glass
theme-manager apply glass
niri validate
cat ~/.config/theme-manager/generated/foot.ini
```

Then run:

```bash
theme-manager watch
```

Edit `glass.toml` and the Noctalia palette independently. Both should reapply the
same active structure profile.

## 6. Remove the old service last

Once the Rust watcher is stable, remove/disable the old Noctalia `watcher.luau`,
widget, and structure pipeline. Use `theme-manager tui` for interactive control.
