# Theme system simplification and portability plan

## Status

Implemented as Theme Manager v0.3. The audited baseline and resolution record
live in `PHASE-0-INVENTORY.md`; byte-compatible v0.2 Niri/Foot output remains
pinned by golden fixtures. Phases 0–5 and 7 are complete. Phase 6 was explicitly
resolved by keeping the CLI as the sole frontend: a TUI remains optional, has no
demonstrated workflow that the CLI lacks, and therefore is not shipped or pulled
into minimal/headless installations.

The final implementation includes the qualified
`stylix-cursors/bibata-modern-ice` library selected by both host bundles. Nix
configures it through upstream `stylix.cursor` and only the GTK/X11 cursor
subtargets, then emits the read-only library and capability inventory consumed
by Theme Manager. All Stylix color and application targets remain disabled, so
Noctalia and the built-in wires retain their documented ownership.

Verification is complete at repository level: the Rust test/lint/format suite,
release package and install checks, Nix source checks, full flake check, and both
host evaluations pass. The pinned Noctalia CLI was also exercised directly for
headless template rendering and for distinct `m3-content`/`muted` results from
one image.

This plan replaces the earlier assumption that Stylix must be the foundation of
the whole theme system. The runtime engine remains independent of every desktop
environment, window manager, desktop shell, and color generator. Stylix and
Noctalia are optional integrations with different responsibilities.

## Decisions

1. Preserve the useful architecture already present in Theme Manager:
   application-independent structure profiles, a normalized semantic color
   palette, and target-specific rendering at the edges.
2. Keep the engine independent of Niri, Hyprland, Noctalia Shell, Quickshell,
   Foot, Stylix, and any future desktop component.
3. Rename and narrow the public concepts:
   - providers become **Color Providers**;
   - configured output integrations become **Application Wires**;
   - a wire uses either a built-in **Target Adapter** or the built-in,
     deliberately constrained **Noctalia Template Adapter**;
   - the word **plugin** disappears unless a real third-party runtime extension
     requirement appears later.
4. Remove the general-purpose Luau plugin platform, discovery, manifests,
   sandbox, resource limits, factories, and registries. They solve a hypothetical
   ecosystem problem rather than the current configuration's needs.
5. Support more than one named color palette. A palette selects a Color Provider
   and its provider-specific scheme or input. An Application Wire uses the
   active Theme Bundle's palette unless it explicitly selects another one.
6. Keep structural appearance independent from color generation. Opacity, blur,
   corner radius, gaps, focus rings, shadows, and similar values belong to a
   Structure Profile, not to Stylix or Noctalia.
7. Allow Noctalia to fill two independent optional roles: a headless Color
   Provider and a renderer for Noctalia Template Wires. Both roles have a
   shell-independent path; a running Noctalia Shell must not be mandatory.
8. Keep Stylix as a separate, optional declarative choice. Use its upstream
   targets directly for rebuild-time application wiring when that is useful. Do
   not wrap, copy, fork, or reimplement Stylix targets inside Theme Manager.
9. Give every setting and generated file exactly one owner. Stylix targets,
   built-in Target Adapters, and Noctalia Template Wires may touch the same
   application only when their settings are disjoint and the application has a
   clean merge or include boundary.
10. Keep the CLI as the complete primary interface. The TUI remains optional and
    must not determine the core architecture.
11. Present one **Theme Bundle** as the user-facing selection without collapsing
    unlike values into a palette. A bundle explicitly selects a Color Palette,
    Structure Profile, and, where configured, named typography, cursor, icon,
    and wallpaper items from named libraries.
12. Make the source of every non-color item explicit. For example, a cursor is
    selected as an item from `stylix-cursors`, not as an unqualified cursor name
    that happens to be delivered by Stylix.
13. Resolve enabled Stylix targets, built-in wires, and Noctalia Template Wires
    into one validated **Capability Plan**. This is an ownership and coverage
    planner, not a composite Color Provider or a renderer that rewrites one
    system's output through another.
14. Define completeness against small typed application contracts. Every
    required capability has exactly one owner; optional capabilities are
    reported but may be absent. Multiple contributors may complete one
    application only across a real, tested merge or include boundary.
15. Report the delivery cadence of every selection and capability as
    `runtime`, `session`, or `rebuild`. A Theme Bundle is one desired appearance,
    but applying it must not pretend that Nix-owned or session-scoped features
    changed live.

## What is already true

The existing implementation is more portable than its current deployment makes
it appear:

- `theme-core` receives a `PaletteProvider` and a list of `Target` values; the
  engine itself contains no Niri, Foot, or Noctalia behavior.
- A palette is already a string-to-string map of semantic color names.
- Structure profiles already describe semantic roles such as `browser`,
  `terminal`, and `compositor` rather than naming applications.
- The Noctalia integration only reads a rendered JSON palette. The Rust engine
  does not communicate with Noctalia Shell.
- Niri and Foot are target implementations at the edge, not engine requirements.

The current deployment is nevertheless specific:

- `modules/user/theme-manager/config.toml` selects Noctalia, Niri, and Foot;
- the Home Manager module knows how to include Niri and Foot fragments;
- the current Noctalia Home Manager wiring creates the JSON bridge only when
  `desktopShell == "noctalia"`;
- the package ships a Niri/Foot default configuration.

The migration should preserve the first list and remove the accidental coupling
in the second list.

## Terminology

Use these terms consistently in Rust, Nix, configuration, documentation, and
the CLI.

### Color Provider

A Color Provider obtains or generates colors and normalizes them into Theme
Manager's semantic color map. It does not render application configuration.

Initial providers:

- **Static**: colors declared directly in configuration.
- **JSON file**: a normalized palette generated by another program.
- **Noctalia**: invokes the known `noctalia theme` CLI headlessly from an image or
  precomputed Noctalia theme JSON and reads a rendered normalized palette, or
  reads the normalized bridge emitted by a running Noctalia instance while
  preserving Noctalia provenance.

A Color Provider may declare source paths that must be watched. There will be no
arbitrary command provider initially; process execution is limited to explicit,
built-in integrations such as Noctalia.

### Color Palette

A named configuration of a Color Provider. It selects the provider and contains
only the options needed to obtain one palette, for example an image, a JSON path,
or a Noctalia generation scheme.

Use `scheme` or `variant` for values such as `m3-content` and
`m3-tonal-spot`. Do not call them `material`: the existing profiles use
`material` to mean structural surface behavior such as `glass` or `solid`.
The migration found that structural field unused and removed it. If a real
adapter later needs the concept, call it `surface_style` so the two concepts
cannot be confused.

### Appearance Library

A named, typed catalog of non-color appearance items supplied through one
explicit integration. Initial useful library kinds are:

- **Stylix Cursor Library**: repository-declared cursor items delivered through
  `stylix.cursor`, each with a Nix package, cursor name, and size;
- **Stylix Typography Library**: repository-declared font sets delivered through
  `stylix.fonts`, including serif, sans-serif, monospace, emoji, and sizes;
- **Stylix Icon Library**: repository-declared icon themes delivered through
  `stylix.icons`;
- **Stylix Wallpaper Library**: repository-declared images and scaling modes
  delivered through `stylix.image` and `stylix.imageScalingMode`.

The user always selects both a library and an item, for example
`library = "stylix-cursors", item = "bibata-modern-ice"`. Do not expose an
unqualified `cursor = "bibata-modern-ice"` and infer Stylix from ambient host
configuration.

An Appearance Library is not a runtime package repository, discovery service,
or download catalog. Its items are declared in the local Nix configuration so
package values remain typed Nix values and dependencies remain pinned. The Home
Manager module may emit a read-only runtime inventory containing stable item
IDs, display metadata, effective values, source, and delivery cadence; runtime
TOML never evaluates package expressions.

Keep separate libraries when the delivery integration differs. A future
repository-owned runtime cursor library must not silently reuse the
`stylix-cursors` name. Library names make provenance part of configuration and
of `theme-manager check` output.

Selecting an item does not by itself prove system-wide delivery. The host's Nix
configuration declares the consumers for which coverage is required, such as
GTK, X11, Niri, or a particular application. The generated inventory records
which of those consumers each Stylix target actually covers. A selected cursor
is complete only when it is installed and every required cursor consumer has
one selection owner; unknown coverage is reported, not treated as global
success.

### Theme Bundle

A user-facing named selection that groups coherent appearance intent without
erasing component boundaries. A Theme Bundle selects:

- exactly one Color Palette;
- exactly one Structure Profile;
- optionally one item from each configured Typography, Cursor, Icon, and
  Wallpaper Library.

The bundle stores references, not copied colors or asset data. A wallpaper used
as a displayed background and an image used as a palette-generation input are
separate references even when they point to the same declared image. Their
relationship must be explicit rather than inferred from a filename.

A bundle that references any `rebuild` item must be visible to Nix evaluation.
Declare it in the local Nix module or in a repository-owned data file read by
that module; Home Manager emits the corresponding immutable runtime bundle
definition. Do not independently hand-author its Stylix selection in Nix and
its library references in mutable runtime TOML. The Nix-selected declarative
theme determines which rebuild-owned items are effective.

Runtime switching is permitted between bundles whose rebuild-owned selections
already match the effective inventory. Selecting a bundle with a different
cursor package, font set, icon package, wallpaper, or other rebuild-owned item
requires changing the Nix declarative theme and activating it first. Runtime-only
bundles that reference no declarative assets may remain entirely in runtime
configuration.

Initially, non-color bundle selections are declarative assertions and status
inputs. Theme Manager applies runtime-capable parts and reports `session` parts
that await restart. If a selected `rebuild` item differs from the effective
Nix-generated inventory, apply fails before runtime writes and tells the user a
NixOS or Home Manager rebuild is required; Theme Manager never invokes that
rebuild automatically. Add runtime selection for an asset class only when all
of these are true:

- the selected item is already installed declaratively;
- the platform exposes a supported, deterministic runtime selection mechanism;
- the adapter can state which toolkits or sessions it covers;
- switching does not compete with the active Stylix target.

### Delivery Cadence

Every selected item and capability reports when it can become effective:

- **runtime**: Theme Manager can apply it immediately;
- **session**: it is configured, but complete effect requires a new graphical
  session or application restart;
- **rebuild**: it changes only through NixOS/Home Manager evaluation and
  activation.

Cadence is not an implementation detail. It is shown by `check`, `status`, and
apply results so a Theme Bundle never implies false system-wide atomicity.

### Structure Profile

Application-independent appearance intent such as:

- background opacity;
- blur enabled/disabled and, if required, blur intensity;
- corner radius;
- inner and outer gaps;
- focus-ring width, colors, and opacity;
- shadow strength and color;
- semantic role styles for `browser`, `terminal`, `compositor`, or other roles;
- structural surface style such as `glass` or `solid`.

A Structure Profile never names Noctalia, Stylix, Niri, Hyprland, Foot,
Firefox, or Quickshell.

### Resolved Appearance

The validated combination of one Structure Profile and one normalized Color
Palette. It contains concrete numbers, booleans, and colors and is independent
of any target configuration syntax.

It remains the color-and-structure input to runtime adapters. Cursor packages,
fonts, icon packages, and wallpapers do not enter Resolved Appearance; they
remain typed Theme Bundle selections handled by their declared library and
delivery integration.

### Target Adapter

Built-in implementation code that translates a Resolved Appearance into an
application's native fragment format. Examples are Niri KDL, Hyprland
configuration, Foot INI, or JSON consumed by a custom Quickshell.

A normal Target Adapter:

- declares the fixed filename or filenames it owns beneath the generated
  directory;
- validates the semantic color roles and structure fields it consumes;
- renders text in memory;
- cannot select arbitrary paths, discover other adapters, access the network, or
  launch arbitrary commands;
- does not contain host-specific app-ID mappings unless those mappings are
  intrinsic to the target format.

The `noctalia-template` adapter is the one deliberate delegation adapter. It
invokes only the known Noctalia CLI and renders application colors from a
Noctalia-backed Color Palette. It does not consume structural fields or become
an arbitrary command adapter. Its contained headless output and explicitly
delegated shell-owned output rules are defined separately below.

### Application Wire

A configured application integration. Most wires are instances of a typed
Target Adapter. A Noctalia Template Wire instead selects the constrained
`noctalia-template` adapter and delegates color rendering to Noctalia's template
engine. Both forms connect machine-specific facts to portable appearance data:

- which adapter to use;
- which semantic roles the target consumes;
- application IDs or window match expressions;
- whether a compositor or the application owns opacity and blur;
- an optional Color Palette override;
- enable/disable state;
- adapter-specific settings that have a demonstrated current use.

An adapter declares the capabilities it actually provides and the output or
setting boundary through which it provides them. Configuration may enable,
select, or narrow those capabilities, but must not invent capability claims for
an adapter. This prevents a user-authored `provides` list from making an
incomplete renderer appear complete.

For example, the Niri adapter is implementation code. A wire that maps
`^firefox$` to the `browser` role and `^(foot|footclient)$` to the `terminal`
role is host/runtime configuration.

### Noctalia Template Wire

An Application Wire that uses Noctalia's existing template engine instead of
reimplementing an application's color syntax in Rust. It is appropriate for
application-native color files already covered by a Noctalia built-in, pinned
community, or repository-owned user template.

A Noctalia Template Wire:

- must select a Noctalia-backed Color Palette because Noctalia templates may
  consume its full Material and tonal-palette data model, not merely Theme
  Manager's smaller normalized map;
- owns application colors only, unless a reviewed template explicitly declares
  a larger ownership boundary;
- does not consume corner radius, blur, gaps, opacity, or other Structure Profile
  fields;
- may coexist with a compositor Target Adapter that owns only those structural
  fields;
- must not coexist with a Stylix target that writes the same application colors
  or theme file;
- identifies an audited template or catalog ID rather than accepting an
  arbitrary renderer command.

This is still a wire, not a plugin API. Noctalia supplies the template language
and renderer; Theme Manager supplies selection, ownership validation, and apply
coordination.

### Stylix target

An upstream NixOS or Home Manager module that applies rebuild-time colors,
fonts, opacity, cursors, wallpaper, or other appearance settings to a supported
application. It is conceptually a premade declarative wire, but it remains owned
and executed by Stylix rather than Theme Manager.

For planning, a selected Stylix target contributes explicit capability claims
from a Nix-generated inventory. Theme Manager does not introspect Stylix
internals at runtime or wrap the target. The inventory records only the target
features and ownership boundaries verified by this configuration.

### Application Contract

A small typed declaration of what this configuration means by complete support
for one application. It contains required and optional capabilities, for
example:

```text
foot.required = [colors, font, background_alpha, reload]
foot.optional = [cursor_color, bell_color, padding]
```

Contracts describe desired outcomes, not every setting the application has.
Start with capabilities used by the two hosts and add one only when a real
configuration needs it. Absence of an optional capability is visible but does
not fail validation; absence or duplicate ownership of a required capability
does.

Do not use a minimal contract to hide useful features. During Phase 0, record
each capability discovered in current output, selected Stylix targets, Noctalia
templates, and application documentation as `required`, `optional`, or
`excluded` with a short reason. Adapter and generated target metadata distinguish
capabilities they **support** from claims they currently **provide**. `check`
shows supported-but-unselected optional capabilities, so a useful Stylix feature
does not disappear merely because the initial bundle omitted it. This is a
review ledger, not a promise to model every upstream option forever.

Use coarse capabilities that correspond to real ownership boundaries. Initial
vocabulary may include:

- global: palette, cursor installation, cursor selection, font installation,
  typography selection, icon installation, icon selection, wallpaper;
- application: colors, font, background alpha, application structure, reload;
- compositor: window opacity, blur, corner radius, gaps, focus ring, shadow.

Do not create a free-form capability namespace, a dependency solver, or one
capability per application key.

### Capability Claim and Capability Plan

A Capability Claim records a typed capability, its scope, owner, source,
delivery cadence, and concrete merge/output boundary. Claims come from built-in
adapter metadata, the Nix-generated Stylix inventory, or the constrained
Noctalia Template Adapter; they are not arbitrary assertions in runtime
configuration.

The Capability Plan combines the claims selected by the active Theme Bundle and
enabled integrations, then validates:

1. every required application and global capability has exactly one owner;
2. optional missing capabilities are reported;
3. no two contributors own the same file, setting group, or exclusive
   capability;
4. contributors to one application meet only at a declared and tested merge or
   include boundary;
5. the effective source and cadence of every capability are visible;
6. every referenced library item is declared, installed where necessary, and
   compatible with its delivery integration;
7. selected assets cover every host-declared consumer in their required scope.

The plan is an internal value shared by `check`, `status`, and `apply`. It is
not a fourth wire choice, a composite provider, a plugin API, or a universal
intermediate configuration format.

### Frontend

An interface over the same runtime operations. The CLI is primary. The optional
TUI calls library functions and contains no separate color, resolution, or
rendering logic.

## Desired architecture

```text
                         COLOR INPUTS

       static colors       JSON file       Noctalia CLI
             |                 |                 |
             +-------- Color Providers ---------+
                               |
                        named Color Palettes
                               |
             +-----------------+------------------+
             |                                    |
             | normalized colors                  | Noctalia-backed only
             v                                    v
 Structure Profile ------> appearance resolver    Noctalia Template Wires
                               |                           |
                      Resolved Appearance                  v
                               |                   application color themes
                    built-in adapter wires
                               |
                +--------------+---------------+
                |              |               |
                v              v               v
          Niri adapter    Foot adapter    Quickshell adapter
                |              |               |
                v              v               v
             niri.kdl       foot.ini       appearance.json

       CLI -------- shared runtime operations -------- optional TUI
```

Stylix is deliberately beside this pipeline rather than inside it:

```text
                        NIX EVALUATION

       Stylix palette/fonts/cursor/image/opacity
                              |
                       upstream Stylix targets
                              |
               immutable Home Manager/NixOS output

                         RUNTIME PIPELINE

       selected Theme Bundle palette + Structure Profile
                              |
                       Theme Manager wires
                              |
                  exclusively owned mutable fragments
```

Stylix may optionally export its evaluated colors as a normalized JSON Color
Palette for Theme Manager. Theme Manager must not call Stylix targets or treat
them as runtime adapters.

The user-facing selection and validation layer sits above both pipelines:

```text
                           Theme Bundle
             palette + structure + explicit library items
                                  |
                                  v
                         desired capabilities
                                  |
          +-----------------------+-----------------------+
          |                       |                       |
          v                       v                       v
 Nix-generated Stylix      built-in adapter       Noctalia template
 target/library inventory       claims                 claims
          |                       |                       |
          +--------------- Capability Plan -------------+
                                  |
                    coverage, collision, boundary,
                         source, cadence, status
                                  |
              +-------------------+-------------------+
              |                   |                   |
              v                   v                   v
           rebuild             runtime             delegated
            domain              domain           Noctalia domain
```

This produces one validated desired configuration, not one global transaction.
All inputs are validated before Theme Manager-owned writes; each owned file is
atomically replaced. Nix activation and shell-driven Noctalia remain separate
transaction domains and their status is reported separately. The planner must
not claim rollback or atomicity across those domains.

## Application wiring choices

Applications have three explicit contribution sources. They are alternatives
for an owned capability, setting group, or file, not necessarily alternatives
for the whole application and never layers that rewrite each other's output.

| Choice | Update time | Palette input | Best use | Structural fields |
| --- | --- | --- | --- | --- |
| Stylix target | Nix rebuild/activation | Stylix palette | broad declarative coverage already supported upstream | only those exposed by Stylix |
| Built-in Target Adapter wire | runtime | any normalized Color Palette | typed, validated colors and portable structure such as blur, radius, gaps, and opacity | yes |
| Noctalia Template Wire | runtime | Noctalia-backed Color Palette | existing Noctalia application color templates | no |

Selection is per application and, where there is a clean include boundary, per
setting group. For example, a Noctalia Foot template may own Foot's colors while
a Niri adapter owns the terminal window's blur and corner radius. Stylix's Foot
target must then be disabled because it would compete for the same colors.

Multiple sources may complete one application. For example, Stylix may own
Foot's colors and font, a narrow Foot wire may own background alpha, and a Niri
wire may own window blur and corner radius. That composition is valid only when
the generated settings are disjoint and Foot or the compositor provides a
stable merge/include boundary. The Capability Plan checks the combined Foot
contract rather than requiring one contributor to implement everything.

If Stylix and a custom Foot adapter both emit one monolithic file, capability
labels cannot make them composable. Disable one renderer. Prefer either a small
declarative extension through normal Home Manager options or a complete Foot
adapter that consumes an exported Stylix palette while the Stylix Foot target is
disabled.

Noctalia Template Wires are not a fallback chain after built-in wires. Choose
one color owner deliberately. Prefer an existing audited Noctalia template over
writing a new Rust color renderer; prefer a built-in adapter when the target
needs Structure Profile values or stronger typed validation.

An application is **complete** when all capabilities required by its Application
Contract have exactly one owner. It is **incomplete** when any required
capability is missing and **conflicting** when an exclusive capability, setting,
or file has multiple owners. `check` must show all three states and the source
and cadence of every claim.

## Normalized color contract

Color Providers normalize their native vocabulary into semantic names. The
initial common set should be only what the current profiles and adapters need:

- `background` and `on_background`;
- `surface` and `on_surface`;
- `primary` and `on_primary`;
- `secondary` and `on_secondary`;
- `error` and `on_error`;
- `outline`;
- `shadow`.

Optional terminal roles may be added when a real adapter consumes them, for
example terminal background, foreground, cursor, selection, and ANSI colors.
Do not require every provider to manufacture unused tokens.

The representation remains a map rather than a large universal struct. Every
value must be a validated `#RRGGBB` or `#RRGGBBAA` color. Each Target Adapter
reports a clear error when a required role is absent.

Provider-specific names do not leak into Structure Profiles or Target Adapters.
The provider owns mappings such as:

- Noctalia `primary` -> normalized `primary`;
- Noctalia `surface` -> normalized `surface`;
- a Base16 source's chosen accent -> normalized `primary`;
- a Base16 source's background -> normalized `background` and/or `surface`.

Any Base16-to-semantic mapping must be explicit and tested. There is no uniquely
correct automatic mapping between Base16 and Material 3 roles.

## Proposed runtime configuration

The exact TOML syntax may change during implementation, but the relationships
must remain this direct. For bundles containing rebuild-owned items, this TOML
is Home Manager output from the Nix-owned definition rather than a second
hand-written source of truth:

```toml
profiles_dir = "$XDG_CONFIG_HOME/theme-manager/profiles"
state_file = "$XDG_STATE_HOME/theme-manager/state.toml"
generated_dir = "$XDG_CONFIG_HOME/theme-manager/generated"
appearance_inventory_file = "$XDG_CONFIG_HOME/theme-manager/appearance-inventory.toml"
default_theme = "forest-glass"

[theme_bundles.forest-glass]
structure = "glass"
palette = "wallpaper"
cursor = { library = "stylix-cursors", item = "bibata-modern-ice" }
typography = { library = "stylix-typography", item = "desktop-default" }
icons = { library = "stylix-icons", item = "papirus-dark" }
wallpaper = { library = "stylix-wallpapers", item = "forest" }

[theme_bundles.forest-paper]
structure = "paper"
palette = "wallpaper-muted"
# Explicitly reuse the same Stylix-sourced assets; never infer their source.
cursor = { library = "stylix-cursors", item = "bibata-modern-ice" }
typography = { library = "stylix-typography", item = "desktop-default" }
icons = { library = "stylix-icons", item = "papirus-dark" }
wallpaper = { library = "stylix-wallpapers", item = "forest" }

[application_contracts.foot]
required = ["colors", "font", "background_alpha", "reload"]
optional = ["cursor_color", "bell_color", "padding"]

[application_contracts.firefox]
required = ["colors"]
optional = ["font", "window_structure"]

[color_providers.noctalia]
kind = "noctalia"

[color_palettes.wallpaper]
provider = "noctalia"
source = "image"
image = "$XDG_CONFIG_HOME/background"
scheme = "m3-content"
mode = "dark"

# The same wallpaper may define another named palette with a different tonal
# treatment. Wires can select this palette explicitly.
[color_palettes.wallpaper-muted]
provider = "noctalia"
source = "image"
image = "$XDG_CONFIG_HOME/background"
scheme = "muted"
mode = "dark"

# Optional authority declaration for wires rendered by a running Noctalia.
[color_palettes.noctalia-shell]
provider = "noctalia"
source = "shell-json"
path = "$XDG_CACHE_HOME/theme-manager/noctalia-palette.json"
mode = "dark"

[color_providers.static]
kind = "static"

[color_palettes.fallback]
provider = "static"

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

[application_wires.niri]
adapter = "niri"
enabled = true
global_role = "compositor"

[[application_wires.niri.windows]]
role = "browser"
app_id = "^firefox$"
opacity_owner = "compositor"

[[application_wires.niri.windows]]
role = "terminal"
app_id = "^(foot|footclient)$"
opacity_owner = "application"

[application_wires.foot]
adapter = "foot"
enabled = true
role = "terminal"

# Alternative color owner for Foot. Do not enable this and let the Foot adapter
# or Stylix also render Foot colors.
[application_wires.foot-noctalia-colors]
adapter = "noctalia-template"
enabled = false
palette = "noctalia-shell"
template = "user:foot-colors"
execution = "shell"

# A shell-independent, embedded repository template is rendered only inside
# generated_dir.
[application_wires.normalized-palette]
adapter = "noctalia-template"
enabled = false
palette = "wallpaper"
template = "repository:palette"
output = "noctalia/palette.json"
execution = "headless"

# Optional: this wire intentionally uses a different Color Palette.
[application_wires.custom-shell]
adapter = "quickshell-json"
enabled = false
palette = "fallback"
role = "desktop_shell"
```

The Appearance Library definitions that contain Nix packages live in the local
NixOS/Home Manager module, not in runtime TOML. The exact option shape may
change, but provenance must remain explicit:

```nix
themeManager.appearanceLibraries = {
  stylix-cursors = {
    kind = "stylix-cursor";
    items.bibata-modern-ice = {
      package = pkgs.bibata-cursors;
      name = "Bibata-Modern-Ice";
      size = 24;
    };
  };

  stylix-typography = {
    kind = "stylix-typography";
    items.desktop-default = {
      monospace = {
        package = pkgs.nerd-fonts.jetbrains-mono;
        name = "JetBrainsMono Nerd Font";
      };
      sansSerif = {
        package = pkgs.inter;
        name = "Inter";
      };
      serif = {
        package = pkgs.noto-fonts;
        name = "Noto Serif";
      };
      emoji = {
        package = pkgs.noto-fonts-color-emoji;
        name = "Noto Color Emoji";
      };
      sizes = { applications = 12; desktop = 10; popups = 10; terminal = 12; };
    };
  };

  stylix-icons = {
    kind = "stylix-icons";
    items.papirus-dark = {
      package = pkgs.papirus-icon-theme;
      dark = "Papirus-Dark";
      light = "Papirus";
    };
  };

  stylix-wallpapers = {
    kind = "stylix-wallpaper";
    items.forest = {
      image = ./wallpapers/forest.png;
      scalingMode = "fill";
    };
  };
};

themeManager.declarativeTheme = "forest-glass";

themeManager.requiredAppearanceConsumers = {
  cursor = [ "gtk" "niri" ];
  icons = [ "gtk" ];
  typography = [ "fontconfig" "foot" ];
  wallpaper = [ "niri" ];
};
```

The module translates the selected item into the corresponding public Stylix
options and emits an inventory for runtime validation. A generated cursor entry
contains at least:

```toml
[libraries.stylix-cursors]
kind = "stylix-cursor"
source = "stylix.cursor"
installation_cadence = "rebuild"

[libraries.stylix-cursors.items.bibata-modern-ice]
name = "Bibata-Modern-Ice"
size = 24
installed = true
effective = true
```

Selection claims record their own cadence separately because installation and
selection are distinct and a toolkit or session may realize the configured
cursor later than Nix installs its package.

The inventory also records audited Stylix target claims, such as whether the
enabled Foot target provides colors, font, background alpha, or some subset.
Do not assume that enabling a target means it fulfills the complete application
contract. Regenerate this inventory during Nix evaluation from local declared
metadata; do not scrape generated files or inspect Stylix module internals at
runtime.

Full output paths are not configurable for Theme Manager-owned output. Each
built-in adapter owns known filenames beneath `generated_dir`, such as
`niri.kdl`, `foot.ini`, or `quickshell.json`. A headless Noctalia Template Wire
may choose only a relative output beneath `generated_dir`; absolute paths,
parent traversal, and symlink escapes are rejected. This makes ownership
obvious and removes force-overwrite negotiation over arbitrary user files.

A shell-driven Noctalia Template Wire is an explicit delegation exception:
Noctalia owns its documented output paths and apply/undo hooks. Theme Manager
records and collision-checks that ownership but does not pretend those files are
inside its generated directory or transaction. Enabling such a wire therefore
requires an explicit template ID and a successful `theme-manager check` that
shows its known outputs and conflicts. Theme Manager configuration never accepts
raw hook commands, dynamic output commands, or unrestricted destination paths.

Input, profile, state, and generated directory roots follow XDG defaults and may
be overridden explicitly. An override of `generated_dir` changes the owned root;
it does not allow individual wires to escape it. Reject symlink or canonical
path escapes outside that root.

## Resolution and application behavior

Applying a Theme Bundle performs these steps:

1. Load the bundle and resolve its explicitly named Structure Profile, Color
   Palette, and Appearance Library items.
2. Load the Nix-generated Appearance Library and Stylix target inventory. Reject
   unknown libraries, unknown items, source-kind mismatches, and required items
   that are not installed.
3. Collect capability claims from selected Stylix targets, enabled built-in
   wires, Noctalia Template Wires, and selected library items.
4. Build the Capability Plan. Reject duplicate ownership, missing required
   capabilities, unsafe path overlap, and composition without a declared merge
   boundary. Record optional gaps without failing.
5. Compare desired bundle selections with the effective inventory. Reject an
   ineffective `rebuild` selection before changing runtime output. Report a
   configured `session` selection as pending without claiming it is live; do
   not invoke a rebuild or restart automatically.
6. Collect the bundle palette plus any palette overrides selected by enabled
   Application Wires.
7. Load each unique Color Palette once through its Color Provider, then normalize
   and validate every loaded color map.
8. Resolve the bundle's Structure Profile once per unique palette.
9. Pass the appropriate Resolved Appearance to each normal Target Adapter.
10. For each headless Noctalia Template Wire, verify that its palette is
    Noctalia-backed, resolve its audited template, and invoke `noctalia theme`
    against a staging destination. Read the result back before installation.
11. Render or stage every Theme Manager-owned output before modifying a live
    destination.
12. Verify that every Theme Manager-owned path is inside the generated directory
    and that the rendered paths still match the validated Capability Plan.
13. Compare with existing files, skip unchanged output, and atomically replace
    changed files.
14. If shell-driven Noctalia wires are selected, request their separately owned
    render only after Theme Manager-owned output succeeds. Do not run shell IPC
    in headless mode.
15. Persist the active Theme Bundle after Theme Manager-owned output succeeds.
    Record the effective status of each component and report any pending
    `session` work or delegated Noctalia failure separately. A delegated failure
    returns a nonzero result.

An invalid provider result, profile, wire, or Theme Manager-owned render must
leave all Theme Manager-owned last-valid files intact. Cross-file rollback
machinery is unnecessary unless a concrete failure demonstrates that
independently atomic fragments are insufficient. The minimum contract is that
each visible owned file is always individually complete and valid; all
validation and rendering occurs before the first owned write. Noctalia-owned
shell outputs are a separate transaction controlled by Noctalia and cannot be
rolled back by Theme Manager.

Application Wires may ignore profile fields that are irrelevant to their target,
but they must document the fields they consume. They must not silently ignore a
configured mapping that they claim to support.

A Noctalia Template Wire intentionally consumes no Structure Profile fields. It
must say so in `theme-manager check`; structure for that application comes from
a separate, disjoint compositor or application adapter.

## Structural ownership examples

Structure remains portable; adapters decide how the target can express it.

### Terminal role

- Foot maps `background_opacity` to its native alpha setting.
- Niri or Hyprland maps `blur` and `corner_radius` to compositor window rules for
  the terminal's app ID.
- The compositor must not apply whole-window opacity when Foot already owns
  background opacity, unless the wire explicitly requests that effect.

An example complete Foot contract may resolve as:

| Foot capability | Owner | Source | Cadence | Boundary |
| --- | --- | --- | --- | --- |
| colors | Stylix Foot target | `stylix-target:foot` | rebuild | Stylix-owned color settings |
| font | Stylix Foot target | `stylix-target:foot` | rebuild | Stylix-owned font setting |
| background alpha | narrow Foot wire | `theme-manager-wire:foot-alpha` | runtime | dedicated included fragment |
| reload | Foot wire/application | `theme-manager-wire:foot-alpha` | runtime | documented Foot reload mechanism |
| blur and corner radius | Niri wire | `theme-manager-wire:niri` | runtime | compositor window rule |

This is valid only after inspecting the actual generated configuration and
proving that the Foot include does not repeat Stylix-owned colors or fonts. If
that boundary is unavailable, disable the Stylix Foot target and let one
complete Foot adapter own the mutable Foot fragment. The adapter may consume a
normalized palette exported from Stylix; that makes Stylix the color authority,
not a second Foot renderer.

### Browser role

- Niri or Hyprland maps the browser app ID to compositor blur, radius, and
  optional whole-window opacity.
- Browser-native colors remain owned by Stylix, Noctalia templates, or another
  explicit application owner; a compositor wire does not theme browser chrome.

### Custom Quickshell

- A Quickshell adapter emits a stable JSON document containing the resolved
  semantic palette and the role-specific structural values actually consumed by
  the shell.
- The Quickshell configuration reads that file and reloads it using its native
  facilities.
- Theme Manager does not need to know the shell's widget hierarchy.

## Noctalia integration

Noctalia is optional and may fill two separate roles:

1. a Color Provider that turns an image or Noctalia theme JSON into Theme
   Manager's normalized semantic colors;
2. a Noctalia Template Adapter that renders application-native color files.

Enabling one role does not enable the other. Noctalia remains neither the owner
of Theme Manager nor a required desktop shell.

### Noctalia as a Color Provider

The provider uses a small packaged Noctalia template to produce normalized JSON,
validates that JSON, and returns a Color Palette. It does not implicitly render
any application templates. This keeps palette generation usable with built-in
Target Adapters and prevents selecting a provider from unexpectedly changing
application files.

Noctalia supports fixed built-in, community, and custom palette sources as well
as dynamic wallpaper-derived colors. Theme Manager's headless Noctalia provider
uses the dynamic form: an image supplies the source colors, a generation scheme
controls their tonal treatment, and Noctalia expands the result into complete
dark/light Material and terminal color roles. When a running Noctalia uses
`source = "wallpaper"`, it regenerates those colors when the default wallpaper
changes.

The setting Noctalia calls `wallpaper_scheme` and the headless CLI exposes as
`noctalia theme --scheme <name>` is represented as `scheme` on a named Color
Palette. It belongs neither to the Color Provider definition nor to a template
wire. Current supported schemes are:

| Scheme | Intended result |
| --- | --- |
| `m3-tonal-spot` | balanced Material 3 tones anchored on the wallpaper seed; Noctalia CLI default |
| `m3-content` | higher-chroma Material 3 colors for content-forward interfaces |
| `m3-fruit-salad` | playful cross-hue Material 3 accents |
| `m3-rainbow` | full-wheel Material 3 colors for multi-accent layouts |
| `m3-monochrome` | Material 3 roles collapsed to a single hue |
| `vibrant` | saturated, high-contrast custom colors |
| `faithful` | colors kept close to the source image |
| `soft` | source-faithful colors with gentler saturation |
| `dysfunctional` | deliberately off-kilter custom colors |
| `muted` | low-saturation custom colors |

The data flow is deliberately one-way:

```text
wallpaper image
      |
      v
Noctalia scheme (muted, vibrant, m3-content, ...)
      |
      v
Noctalia dark/light native palette
      |
      +----> normalized Color Palette ----> built-in Target Adapter wires
      |
      +----> native tokens ---------------> Noctalia Template Wires
```

Two named Color Palettes may reference the same image with different schemes,
for example `wallpaper` using `m3-content` and `wallpaper-muted` using `muted`.
They are separate selectable palettes, not provider layers or transformations.
Theme Manager resolves only the active Theme Bundle's palette and overrides
needed by enabled wires during an apply.

`mode` selects the dark or light variant exposed to a palette and its templates.
Noctalia's optional pure-black dark treatment is a separate palette option, not
a scheme. Fixed built-in, community, and custom Noctalia palettes already
contain their colors and therefore do not use `wallpaper_scheme`.

### Noctalia as an Application Wire

The `noctalia-template` adapter uses Noctalia's template engine for application
colors. This lets the configuration choose among three sources:

- a built-in template ID shipped by Noctalia;
- a community template that has already been fetched, reviewed, and pinned by
  the Nix configuration;
- a repository-owned user template file.

Initial implementation should support shell-selected built-in IDs and explicit
repository-owned template files. Do not automatically fetch or update community
templates. Add pinned community templates only when a real application requires
one. The active template identity and content source must be visible in
`theme-manager check`.

Use `noctalia theme --list-templates` to inspect the installed catalog. Current
built-in examples include Niri, Hyprland, Foot, Alacritty, Kitty, GTK, Qt,
Starship, Helix, and Btop, but the plan does not hard-code that evolving catalog
as Theme Manager's own support list.

A template wire must use a Noctalia-backed Color Palette. For headless execution,
that palette supplies the image or theme JSON given to the CLI. For shell-driven
execution, it must use the `shell-json` source and acts as an authority assertion:
the normalized bridge and the template render both come from the running shell's
current palette. A shell-driven wire cannot claim an unrelated image-backed
palette override.

Theme Manager must not attempt to reconstruct Noctalia's complete
Material/tonal token model from the smaller normalized color map. That reverse
conversion would be lossy and would make results differ between the provider
and the template renderer.

### Shell-driven mode

When Noctalia Shell is already in use, its user-template mechanism may render a
normalized palette JSON whenever the shell palette changes. Theme Manager uses
the JSON-file Color Provider and watches that file. This preserves the current
bridge behavior.

The same Noctalia configuration may opt into built-in or pinned community
template IDs and declare user templates. Those entries are shell-driven
Noctalia Template Wires. Noctalia owns their application output paths, apply and
undo hooks, reload behavior, and cleanup. The Home Manager module translates the
declared wire selection into Noctalia's `builtin_ids`, `community_ids`, or user
template settings; Theme Manager does not edit Noctalia's runtime-managed
`settings.toml`.

The local Home Manager module is the single source of truth for shell-driven
wire selection. It emits both the Noctalia template registration and Theme
Manager's matching wire/ownership metadata; do not maintain two independent
hand-written ID lists that can drift.

Enabling a built-in ID is explicit. An empty ID list means no built-in templates.
Before activation, the ownership inventory must record each selected template's
output files, hook behavior, and any primary application file its apply script
edits. A shell-driven template cannot be treated as a harmless generated
fragment merely because its template input is declarative.

`noctalia msg templates-apply` is IPC to a running shell. Use it only in this
mode and only after Theme Manager-owned output succeeds. Normal palette changes
may already cause Noctalia to rerender enabled templates, so do not send a second
IPC request unless an explicit Theme Manager apply needs it.

### Headless mode

Noctalia's template renderer is also available directly through its CLI:

```text
noctalia theme <image> -r <palette-template>:<palette-output>
noctalia theme <image> -c <templates.toml>
noctalia theme <image> --builtin-config
noctalia theme --theme-json <theme.json> -r <palette-template>:<palette-output>
noctalia theme --list-templates
```

These commands do not require a running Noctalia Shell process. They do require
the Noctalia package, which may still include shell-related dependencies.

The built-in Noctalia Color Provider and Noctalia Template Adapter may invoke
only this known CLI. They never accept a command name or shell snippet from Theme
Manager configuration.

For a headless Noctalia Template Wire:

- render into a private staging directory first;
- accept only a closed repository template ID whose reviewed bytes are embedded
  in the binary; do not accept an arbitrary template path;
- rewrite or reject outputs that do not resolve beneath `generated_dir`;
- reject `output_path_dynamic`, `pre_hook`, `post_hook`, `undo_hook`, and other
  command-bearing fields in the initial implementation;
- read and validate every staged file before atomically installing it;
- let a local Nix/Home Manager module connect the generated fragment to the
  application and perform any declarative reload setup.

Do not invoke `--builtin-config` blindly: it processes the shipped catalog rather
than expressing Theme Manager's per-wire ownership. A built-in template may be
supported headlessly only when its template input and destination are resolved
to one audited wire without running its catalog hooks. Otherwise use its normal
shell-driven ID or add a small repository-owned template.

This restriction keeps headless execution transaction-like and makes it work
without Noctalia Shell. It intentionally gives up some of Noctalia's hook and
dynamic-path features; those remain available only through an explicitly
delegated shell-driven wire.

### Color and structure ownership

Noctalia Template Wires primarily cover application colors. Niche structural
values remain the responsibility of Structure Profiles and built-in Target
Adapters. A valid combination might be:

- Noctalia's Foot template owns Foot's color theme;
- a minimal Foot adapter owns only background alpha, if the include boundary is
  clean;
- the Niri or Hyprland adapter owns blur and corner radius for Foot's window;
- Stylix's Foot target is disabled.

If Foot cannot merge those fragments without overlapping settings, select one
complete application color/config owner instead. Never render a Noctalia theme,
then patch the same keys with a built-in adapter.

Headless configuration must state what triggers regeneration:

- explicit `theme-manager apply`;
- a watched source image or theme JSON change;
- a wallpaper-selection command that invokes Theme Manager after changing the
  selected image.

The watcher watches parent directories and filters logical filenames so atomic
replacement of the image or JSON does not detach an inode-specific watch.

The packaged static Color Provider must keep Theme Manager usable when Noctalia
is not installed. Enabling a Noctalia Template Wire makes the Noctalia package a
dependency for that host even when Noctalia Shell itself is disabled.

Relevant Noctalia references:

- Template and headless CLI reference:
  <https://docs.noctalia.dev/noctalia/theming/templates/>
- Application template behavior:
  <https://docs.noctalia.dev/noctalia/theming/app-theming/>

## Stylix integration policy

Stylix is optional and independent. It provides substantially more than a
palette: upstream targets translate Stylix colors, fonts, opacity, cursors,
wallpaper, and related values into NixOS or Home Manager application settings.
Those targets are a useful rebuild-time wire catalog.

Theme Manager must not import Stylix internals or convert its target modules into
runtime Application Wires. Stylix's `mkTarget` is intended for modules loaded by
Stylix's own autoload mechanism, not ordinary external imports.

### Explicit Stylix libraries and sources

Use Stylix's useful cross-cutting features, but never hide their provenance
behind generic bundle fields:

| Desired feature | Explicit selection/source | Public Stylix options | Initial cadence |
| --- | --- | --- | --- |
| custom cursor | `library = "stylix-cursors"` | `stylix.cursor.package`, `.name`, `.size` | install: rebuild; selection: target-specific |
| typography | `library = "stylix-typography"` | `stylix.fonts.*` and `.sizes.*` | install: rebuild; selection: target-specific |
| icon theme | `library = "stylix-icons"` | `stylix.icons.enable`, `.package`, `.dark`, `.light` | install: rebuild; selection: target-specific |
| wallpaper | `library = "stylix-wallpapers"` | `stylix.image`, `stylix.imageScalingMode` | target-specific rebuild/session |
| static application colors | `source = "stylix-target:<target>"` | upstream target options | rebuild |
| exported normalized colors | named JSON palette with `source = "stylix-export"` provenance | `config.lib.stylix.colors` emitted as JSON | rebuild |
| Stylix-owned opacity | `source = "stylix-opacity"` | `stylix.opacity.*` through supported targets | rebuild |

The library names are local, stable API names chosen by this configuration; they
do not claim that Stylix publishes a downloadable cursor or font marketplace.
For example, the local `stylix-cursors` library can contain several reviewed,
pinned cursor packages. Selecting one means “use this declared item through the
Stylix cursor integration,” not “search Stylix for a cursor with this name.”

Keep installation and selection as separate capabilities. Nix installs the
package; Stylix or a future supported runtime adapter selects the name and size.
A runtime selector may only choose an already installed library item, and its
claim must state which environments it covers, such as GTK, X11, or a specific
Wayland compositor. Do not claim global cursor switching from one toolkit
update.

Likewise, a font library item includes packages, family names, and sizes; an
icon item includes its package and light/dark names; a wallpaper item includes
its image and scaling mode. These values belong to typed library items, not to
the Color Palette or Structure Profile.

Stylix opacity is different: it is a structural capability contribution, not an
Appearance Library item. It may satisfy a contract only for targets and setting
groups that Stylix demonstrably controls. A runtime Structure Profile must not
also own those opacity settings.

Use Stylix directly when all of the following are true:

- the application has a satisfactory upstream target;
- its appearance may change at rebuild/activation time;
- the target does not conflict with a mutable Theme Manager fragment;
- the selected Stylix palette is acceptable for that application.

Use a small local Nix module when:

- the missing behavior is declarative;
- normal NixOS or Home Manager options can express it;
- runtime switching is unnecessary.

Use a Theme Manager Application Wire when:

- the setting must update without a Nix rebuild;
- it depends on the active runtime Structure Profile;
- it depends on a live Color Provider;
- the target lacks a suitable Stylix module;
- a mutable fragment is the application's supported integration boundary.

Within Theme Manager, choose a Noctalia Template Wire when an audited Noctalia
template already covers the required runtime colors. Choose a normal built-in
adapter when the application needs Structure Profile values, a non-Noctalia
palette, or stricter typed rendering. Do not implement both for the same color
keys merely to provide interchangeable backends.

### Per-setting ownership rules

For every application, record an ownership matrix before enabling any
combination of Stylix, built-in Target Adapters, and Noctalia Template Wires.

Example where a clean include boundary exists:

| Foot setting | Owner |
| --- | --- |
| Base colors and font | Stylix |
| Runtime background alpha | Theme Manager `foot.ini` include |

This is valid only if the Theme Manager fragment contains alpha and does not
also render colors or fonts.

If Theme Manager begins rendering Foot colors, disable the Stylix Foot target
and let Theme Manager own the complete mutable theme fragment.

For applications without a clean merge/include boundary, one system owns the
complete configuration. Do not generate a Stylix file and then patch it at
runtime.

### Palette authority modes

Choose and document one of these modes for each host.

#### Stylix-authoritative static colors

- Stylix owns the canonical rebuild-time palette and its supported targets.
- Export evaluated Stylix colors into Theme Manager's normalized JSON contract.
- Theme Manager uses those colors for runtime structural adapters.
- All applications remain color-consistent.
- Changing colors requires a rebuild/activation.

#### Noctalia-authoritative live colors

- Noctalia produces the live palette.
- Theme Manager-owned outputs update immediately.
- Stylix-owned outputs retain their last rebuild-time palette.
- Any intentional difference in update cadence must be visible in the ownership
  inventory and documentation.

#### Fully live colors

- Noctalia or another runtime Color Provider is authoritative.
- Disable Stylix targets for applications whose colors must update live.
- Use built-in Target Adapter wires or Noctalia Template Wires as the sole color
  owner for those applications.
- Stylix may still own unrelated static applications or system properties where
  delayed color updates are acceptable.

Do not automatically rebuild NixOS/Home Manager every time a live palette
changes. That turns an inexpensive runtime operation into an expensive system
deployment and obscures failure handling.

### Stylix modification order

When Stylix lacks static declarative behavior:

1. Configure an existing Stylix or Home Manager option.
2. Add the smallest local Nix module needed.
3. Submit a generally useful target or fix upstream.
4. Carry a narrowly scoped temporary patch only when necessary.
5. Fork Stylix only if its fundamental architecture becomes incompatible and
   the ongoing rebase cost is explicitly accepted.

Local Stylix target modules must guard configuration on both `stylix.enable` and
their local target enable option. Theme Manager installation, service, and
fragment-wiring modules are not Stylix targets; guard them on Theme Manager and
the relevant application enable options instead.

Relevant Stylix references:

- Configuration, including colors, wallpaper, fonts, and target selection:
  <https://nix-community.github.io/stylix/configuration.html>
- NixOS options, including cursor, icons, font sizes, wallpaper scaling, and
  opacity:
  <https://nix-community.github.io/stylix/options/platforms/nixos.html>
- Module/target guidance:
  <https://nix-community.github.io/stylix/modules.html>
- Stylix repository:
  <https://github.com/nix-community/stylix>

## Stable component boundaries

### Appearance Libraries own

- named, explicitly sourced non-color items;
- typed metadata required by their delivery integration;
- the distinction between package installation and selection;
- source and cadence information emitted into the runtime inventory.

They do not generate colors, render arbitrary application configuration, fetch
packages at runtime, or hide one integration behind another library's name.

### Theme Bundles own

- references to one palette and one structure profile;
- explicit library-and-item references for optional typography, cursor, icons,
  and wallpaper;
- the coherent user-facing name of that selection.

They do not copy component values, declare ownership, or determine whether an
adapter is complete.

### Application Contracts and Capability Planner own

- the small typed required/optional capability vocabulary;
- collection of trusted claims from adapters and generated inventories;
- coverage, duplicate-owner, boundary, source, and cadence validation;
- a single desired-versus-effective plan shared by `check`, `status`, and
  `apply`.

They do not render output, resolve colors, invoke Nix, invent claims from user
configuration, or provide cross-domain rollback.

### Color Providers own

- obtaining or generating colors from declared inputs;
- mapping provider-native roles into the normalized semantic contract;
- validating provider-specific configuration;
- declaring source paths that require watching;
- concise provider-specific errors.

They do not own Structure Profiles, application matching, target rendering,
output paths, or frontend behavior.

### Structure resolver owns

- parsing and validating profiles;
- resolving numeric tiers and semantic color references;
- combining one profile with one palette;
- producing application-independent Resolved Appearance values.

### Target Adapters own

- native output syntax;
- fixed output filenames beneath the generated directory;
- target-specific escaping and validation;
- translation of supported structural fields and normalized colors.
- trusted capability metadata for the exact settings and boundary they render.

### Noctalia Template Adapter owns

- validating that the selected palette is Noctalia-backed;
- resolving an audited template ID or repository-owned template input;
- invoking only the known `noctalia theme` interface;
- staging and validating headless template output;
- rejecting command-bearing headless template fields and unsafe paths;
- clearly reporting when rendering is delegated to a running Noctalia instance.

It does not translate Structure Profile fields, fetch community templates,
invent missing Noctalia tokens, or hide Noctalia-owned hooks and output paths.

### Application Wires own

- selecting the adapter;
- app-ID/window-role mappings;
- selecting an optional palette override;
- selecting a Noctalia template identity and headless or shell-driven execution
  when using the Noctalia Template Adapter;
- declaring which component owns effects such as opacity;
- enabling or disabling an integration.

### Local Nix modules own

- package installation;
- user-service configuration;
- optional includes pointing applications at generated fragments;
- optional Stylix configuration and target selection;
- typed Stylix Appearance Library declarations and translation into public
  Stylix options;
- Nix-visible Theme Bundle declarations when they reference rebuild-owned items,
  selection of the active declarative theme, and emission of runtime bundle
  data;
- generation of the read-only library, effective-value, and audited target
  capability inventory, including the evaluated Stylix revision;
- shell-driven Noctalia template ID registration and pinned template packaging;
- small missing declarative settings.

### Frontends own

- presenting operations and results;
- collecting user choices;
- rendering errors clearly.

Frontends do not parse profiles, load providers, resolve colors, render target
configuration, or write generated files themselves.

## CLI and state

The primary CLI should expose only demonstrated workflows:

```text
theme-manager theme list
theme-manager theme show <theme>
theme-manager structure list
theme-manager palette list
theme-manager library list <library>
theme-manager status
theme-manager apply <theme>
theme-manager check [<theme>]
theme-manager watch
```

`library list stylix-cursors` lists only the declared items from that explicitly
named library and shows their Stylix source and cadence. It does not merge cursor
items from unrelated integrations into a global ambiguous list.

Direct structure and palette overrides may exist as expert preview options, but
the normal apply operation selects a complete Theme Bundle so capability
coverage and non-color intent are not bypassed. Top-level `list` may remain as a
short alias for `theme list` if useful.
Compatibility aliases from the pre-release implementation may be deleted when
they add more code than value; there are no external API consumers to preserve.

Persistent state contains only runtime selections and the minimum data needed
for safe operation:

- active Theme Bundle; its palette, structure, and library references remain in
  configuration rather than being copied into state;
- optionally disabled Application Wires, only if runtime toggling remains a real
  workflow.

Desired-versus-effective library status and capability ownership are recomputed
from configuration and the generated inventory. Do not persist a second mutable
copy that can drift from Nix.

Generated-file ownership hashes, transaction journals, force-overwrite flags,
component registries, and health databases should be removed unless the new
exclusive-directory contract proves insufficient. A small watcher status file
is acceptable only if it is used by `status` or service diagnostics.

The optional TUI initially provides:

- Theme Bundle browsing, with component source and cadence visible;
- optional Structure Profile, Color Palette, and explicitly named Appearance
  Library browsing as detail views;
- active desired/effective selection and capability coverage display;
- apply/preview when preview is cheap;
- clear validation and provider errors;
- keyboard navigation, optionally including Vim-style keys.

It may be a Cargo feature or separate package, whichever produces the smaller
clean boundary after the runtime library is stable.

## Watcher and service behavior

Watch mode observes only inputs that can affect active outputs:

- runtime configuration;
- the profile directory and active Theme Bundle;
- the Nix-generated appearance and Stylix target inventory;
- state changes made by another frontend;
- active Color Provider source paths;
- a shell-driven normalized palette JSON;
- headless Noctalia image or theme JSON inputs.

Watch parent directories rather than replaceable files and filter relevant
filenames. Debounce duplicate filesystem events. After any event:

1. reload typed configuration;
2. reload the active Theme Bundle and generated inventory;
3. rebuild and validate the Capability Plan;
4. load all palettes required by enabled wires;
5. render and validate every output;
6. replace only changed files.

Missing provider input at cold start is a waiting state, not a reason to crash in
a restart loop. Log one concise error, keep last-valid output, continue watching,
and retry on a relevant event or bounded periodic fallback.

The Home Manager user service starts with the graphical session, logs through
the journal, restarts only after an unexpected process failure, and does not
require a specific compositor or shell service.

## Plugin policy

There is no runtime plugin API in the replacement.

The current Luau system is removed rather than renamed. `ApplicationWire` does
not mean “plugin”; it is typed configuration selecting a built-in adapter. The
Noctalia Template Adapter delegates rendering to one installed program through a
fixed interface; it is not a general script host, command provider, or adapter
discovery mechanism.

Before introducing any future extension mechanism, require all of the following:

1. A real external user and at least two implementations need the same boundary.
2. A built-in adapter, local Nix module, standalone executable, or compile-time
   Cargo feature is insufficient.
3. The maintenance cost of a versioned public API is accepted.
4. Filesystem, process, and network authority can be constrained appropriately.

Prefer an external program producing normalized JSON over a scripting VM inside
Theme Manager.

## Migration plan

Each phase must leave both hosts evaluable and the currently selected desktop
usable. Do not perform a flag-day rewrite.

### Phase 0: inventory, ownership, and output snapshots

- List every Theme Manager behavior actually used on `homer` and `bart`.
- Inventory every current appearance owner, including Noctalia templates,
  hard-coded application themes, Niri settings, Foot settings, Firefox
  integration, Starship, cursor, fonts, icons, wallpaper, opacity, and any
  selected Stylix targets.
- Mark each setting as one of:
  - optional Stylix target;
  - local declarative Nix setting;
  - runtime Color Provider input;
  - built-in Target Adapter output;
  - headless or shell-driven Noctalia Template Wire output;
  - unused and removable.
- Record exact current Niri and Foot output as golden fixtures.
- Record which fields in `glass.toml` and `paper.toml` are actually consumed.
- Identify unused fields such as structural values that no adapter renders.
- Draft the smallest required/optional Application Contract for each configured
  application. Record capability, owner, source, cadence, output/setting
  boundary, and whether the claim is verified by generated output.
- Classify every useful capability found during the audit as required, optional,
  or explicitly excluded with a reason. Record supported-but-unselected
  capabilities separately from currently provided claims.
- Identify every real merge/include boundary before planning multi-contributor
  application support. Treat an unverified boundary as non-composable.
- Decide the initial host palette authority mode.
- Freeze features in the old Luau architecture.

Deliverable: reviewed ownership and capability matrices, initial application
contracts, and fixtures proving what must survive.

### Phase 1: establish optional Stylix boundaries

- Decide whether either host will use Stylix now; Stylix is not mandatory for
  the runtime migration.
- If selected, add the upstream flake input and appropriate NixOS/Home Manager
  modules.
- Start with explicit target selection rather than broad automatic ownership.
- Enable only targets whose owned generated result is wanted.
- Disable targets that conflict with live Theme Manager colors or fragments.
- Declare any selected cursor through an explicitly named Stylix Cursor Library,
  for example `stylix-cursors/bibata-modern-ice`; do the same for selected
  typography, icons, and wallpaper.
- Keep bundles containing rebuild-owned items and the selected declarative theme
  in Nix-visible configuration, then emit their runtime representation; never
  maintain parallel Nix and mutable-TOML selections.
- Translate library items only through public Stylix options and emit their
  source, installed/effective status, and cadence into a generated inventory.
- Emit audited capability claims for each selected Stylix application target;
  do not equate target enablement with complete contract coverage.
- Verify the chosen static palette, fonts, cursor, icons, wallpaper, opacity,
  and application targets through a rebuild and login.
- Record every allowed rebuild/session/runtime cadence difference.

Deliverable: Stylix is either absent by choice or independently useful without
being a Theme Manager dependency.

### Phase 2: introduce the typed vocabulary without changing behavior

- Rename public concepts from provider/target/plugin to Color Provider, Color
  Palette, Target Adapter, and Application Wire. Introduce Appearance Library,
  Theme Bundle, Application Contract, Capability Claim, and Capability Plan
  with the narrow meanings defined above.
- Introduce typed configuration for named providers, palettes, bundles,
  contracts, libraries, and wires.
- Rename structural `material` to `surface_style`.
- Keep a temporary configuration migration path only long enough to switch both
  hosts in one phase.
- Preserve current Niri/Foot renders and static/Noctalia palette behavior.
- Reject unknown fields at every typed boundary.

Deliverable: both hosts use the new vocabulary and produce byte-equivalent
current fragments.

### Phase 3: replace plugins with minimal built-ins

- Implement only the Static, JSON-file, and headless Noctalia Color Providers
  required by the inventory.
- Implement only the Niri and Foot Target Adapters currently required.
- Keep adapter construction as a small enum/match, not a registry or factory.
- Have built-in adapters expose trusted typed capability and boundary metadata.
- Build one Capability Plan from adapter claims and the Nix-generated inventory;
  fail on missing required coverage or duplicate ownership and report optional
  gaps.
- Fix adapter output filenames beneath one generated directory.
- Resolve each unique palette once and support per-wire palette overrides.
- Render all enabled outputs before writing any of them.
- Add focused tests for normalization, resolution, Niri rendering, Foot
  rendering, path containment, output collision, and one end-to-end apply.

Deliverable: current live appearance works without Luau, discovery, manifests,
or extension factories.

### Phase 4: headless Noctalia and watcher

- Package the normalized Noctalia palette template.
- Support image input and precomputed theme JSON input through `noctalia theme`.
- Map a palette's `scheme` directly to Noctalia's `--scheme` option, defaulting
  explicitly to `m3-tonal-spot` when omitted.
- Validate the supported Noctalia scheme names and report the palette name and
  invalid value without changing last-valid output.
- Allow multiple named palettes to share one watched image while selecting
  different schemes; do not model schemes as provider instances or transforms.
- Ensure the direct CLI path works while Noctalia Shell is stopped.
- Retain shell-driven JSON input as an optional mode for hosts that run the
  shell.
- Add the constrained `noctalia-template` adapter.
- Support repository-owned headless templates through staged `-r` or audited
  `-c` rendering with command-bearing fields disabled.
- Support explicit shell-driven built-in template IDs through the Noctalia Home
  Manager configuration, with their output and hook ownership inventoried.
- Do not fetch community templates automatically; package a reviewed, pinned
  copy before selecting one.
- Watch parent directories, debounce events, and reapply the active selection.
- Preserve last-valid files for missing, partial, or invalid inputs.
- Add the shell-independent Home Manager user service.

Deliverable: live palette changes and at least one Noctalia Template Wire work
with Noctalia Shell stopped, while selected shell-driven IDs remain available as
an explicitly delegated alternative.

### Phase 5: portable wiring

- Replace unconditional Niri/Foot assumptions in the Home Manager module with
  separate opt-in include modules guarded by the corresponding application.
- Keep the runtime engine functional with zero Niri wires and zero Noctalia
  providers.
- Add a Hyprland, another compositor, or custom Quickshell adapter only when that
  environment is actually adopted and its required output is known.
- For each application capability, choose exactly one Stylix target, built-in
  adapter, or Noctalia template owner. Permit multiple contributors to complete
  one Application Contract only where their settings and files are provably
  disjoint and a real merge/include boundary exists.
- Prefer native application settings or includes over editing complete user
  configuration files.
- Upstream generally useful static declarative support to Stylix when
  appropriate; keep runtime-only behavior in Theme Manager.

Deliverable: changing compositor or shell means changing wires/adapters, not the
core data model.

### Phase 6: optional TUI

Implementation decision: no TUI is shipped in v0.3. The complete CLI covers the
normal workflow, and retaining a second frontend would add dependencies and
state without a demonstrated user need. The requirements below remain the gate
if a TUI is added later.

- Verify the CLI covers normal operation first.
- Adapt the TUI to Theme Bundles and their Structure Profile, Color Palette,
  explicit Appearance Library items, Capability Plan, and Application Wires.
- Show desired versus effective values and `runtime`, `session`, or `rebuild`
  cadence; do not present a pending declarative item as live.
- Reuse runtime library operations directly.
- Remove plugin/component management screens.
- Keep Ratatui and Crossterm out of minimal/headless installations when
  packaging this separately is simpler.

Deliverable: the optional TUI is a replaceable frontend over the same API.

### Phase 7: delete obsolete infrastructure

After parity is verified on both hosts, remove:

- the Luau dependency and extension crate;
- plugin discovery directories and manifests;
- the provider and target factory registry;
- generic extension sandboxing and resource limits;
- plugin validation and listing commands;
- packaged example plugins and API definition files;
- component commands used only to toggle plugin instances, unless runtime wire
  toggling is retained as a demonstrated workflow;
- unrestricted or absolute output-path configuration;
- ownership hashes, force overwrite, transaction journals, rollback code, and
  health machinery that the exclusive generated-directory contract makes
  unnecessary;
- stale tests and documentation for the deleted architecture;
- compatibility aliases with no real external users.

Rename any remaining filesystem directories to `adapters/`, `integrations/`, or
another accurate term. Do not merely rename `plugins/` to
`application-wires/`: wires are configuration, while adapters are code.

Deliverable: the repository contains only the portable core, built-in adapters,
current configuration, and tests for the new architecture.

## Verification

Minimum automated and manual checks before deleting the old implementation:

- `cargo test` succeeds for the simplified workspace.
- `nix flake check` succeeds.
- both host configurations evaluate.
- every non-color Theme Bundle item names both its Appearance Library and item;
  an unqualified cursor, font, icon, or wallpaper reference is rejected.
- `stylix-cursors/bibata-modern-ice` translates to the expected
  `stylix.cursor` package, name, and size, and the generated inventory reports
  its source, installation state, effective state, and cadence.
- cursor selection is not reported complete until the inventory has exactly one
  owner for every host-declared cursor consumer; removing GTK or Niri coverage
  reports the precise missing scope.
- an unknown library, unknown item, or item whose kind does not match the bundle
  field fails before runtime output changes.
- `check` distinguishes capabilities an adapter or selected Stylix target
  supports from those it currently provides, and lists supported-but-unselected
  optional features.
- every capability found in the Phase 0 feature audit is required, optional, or
  excluded with a recorded reason; none silently disappears during migration.
- a Theme Bundle whose selected `rebuild` item is not effective fails before
  runtime output changes and reports that Nix activation is required.
- bundles containing rebuild-owned items have one Nix-visible source of truth;
  the emitted runtime definition and effective inventory cannot be configured
  independently.
- runtime switching succeeds between bundles sharing the same effective
  declarative items and fails safely when a bundle requires different ones.
- a configured `session` item that awaits restart is reported as pending rather
  than live.
- the runtime package works with the static provider and no Noctalia package.
- the engine loads with no Niri wire configured.
- a headless Noctalia CLI render succeeds while Noctalia Shell is not running.
- the same image rendered through `m3-content` and `muted` produces two valid,
  independently selectable palettes with different normalized colors.
- omitting a Noctalia wallpaper scheme uses `m3-tonal-spot`, while an unknown
  scheme fails with the provider and palette names and preserves last-valid
  output.
- changing the watched wallpaper regenerates the active wallpaper-derived
  palette without changing its selected scheme.
- a headless Noctalia Template Wire renders into staging and atomically installs
  only beneath `generated_dir`.
- a headless template containing hooks, dynamic output commands, an absolute
  destination, `..`, or a symlink escape is rejected before live output changes.
- a Noctalia Template Wire rejects a Static or normalized JSON palette that lacks
  the native Noctalia token model.
- a shell-driven template wire rejects an image-backed palette and requires the
  matching Noctalia `shell-json` authority source.
- a selected shell-driven built-in template ID appears in the generated Noctalia
  configuration and its known output/hook ownership appears in
  `theme-manager check`.
- the same application color file cannot be owned by Stylix, a built-in adapter,
  and a Noctalia Template Wire simultaneously.
- a complete Foot plan may combine verified disjoint Stylix color/font claims,
  a narrow Foot alpha fragment, and Niri window structure, satisfying every
  required Foot capability exactly once.
- removing the alpha owner makes that Foot plan incomplete and identifies
  `background_alpha` as missing; removing an optional capability reports a gap
  without failing.
- two contributors claiming Foot colors make the plan conflicting even if they
  write different files.
- two contributors that claim disjoint capabilities but write one monolithic
  Foot configuration are rejected for lacking a safe merge boundary.
- capability claims cannot be added or expanded through untrusted runtime
  configuration.
- shell-driven Noctalia JSON changes reapply when that mode is enabled.
- Static and Noctalia providers normalize to the same required semantic
  contract.
- applying `glass` and `paper` produces the intended Niri structure.
- new Foot windows receive the intended background alpha.
- a wire can select a non-default palette.
- two wires selecting the same palette load it only once per apply.
- a missing required semantic color reports the provider, palette, wire, and
  missing role.
- an invalid provider result, profile, or wire leaves all last-valid files
  intact.
- a missing palette at cold start keeps the watcher alive and recovers when the
  file appears.
- atomic replacement of a watched image, profile, config, or palette file still
  triggers an update.
- duplicate event bursts result in one logical reapply.
- reapplying identical input does not rewrite output files.
- restarting the watcher preserves and reapplies the active Theme Bundle and
  recomputes its component references and Capability Plan from configuration.
- attempts to escape the generated directory through `..` or symlinks fail.
- two wires cannot claim the same generated path.
- the runtime tool never writes into the Nix store or a Home Manager-generated
  immutable file.
- Stylix-disabled and Stylix-enabled host evaluations both work.
- when Stylix and Theme Manager touch the same application, a test or inspected
  generated configuration confirms their setting ownership is disjoint.
- `check`, `status`, and `apply` report the same capability owners, sources,
  boundaries, and cadences from one Capability Plan implementation.
- Theme Manager-owned output failure preserves last-valid runtime files;
  delegated Noctalia and Nix domains are reported separately without claiming
  cross-domain rollback.
- the system works without the TUI installed.

## Non-goals

- A public third-party plugin ecosystem.
- Runtime-loaded native or scripted extensions.
- A general command-execution provider.
- Reimplementing Stylix's declarative application coverage.
- Wrapping Stylix targets in Theme Manager abstractions.
- Forking Stylix.
- A universal template language.
- Automatically rebuilding NixOS/Home Manager for every live palette change.
- Absolute or generated-root-escaping output paths selected by adapters or
  headless wires.
- Network access in Theme Manager, including automatic Noctalia community
  template discovery or updates.
- A Theme Manager marketplace or remote catalog. Noctalia community templates
  remain a Noctalia feature and must be reviewed and pinned before use here.
- Treating an Appearance Library as a package search service or automatically
  discovering all cursors, fonts, icons, or wallpapers in Nixpkgs.
- Putting cursors, fonts, icons, wallpaper, or package references into the Color
  Palette.
- A free-form capability namespace, general dependency solver, or capability per
  application setting.
- Global atomic application across Nix activation, Theme Manager-owned files,
  and delegated Noctalia hooks.
- Runtime switching for an asset merely because Stylix can configure it at
  rebuild time.
- Automatic semantic mappings for every color system.
- Solving compositors or shells before they are actually selected.
- Keeping pre-release APIs solely for compatibility.

## Risks and responses

### “Application Wire” becomes a renamed plugin system

Keep wires as typed data and adapters as built-in code. No discovery, manifests,
versioned script API, or runtime loading.

### Stylix and Theme Manager both own one setting

Maintain the per-application ownership matrix. Use separate optional includes
only for disjoint settings. Otherwise disable one owner for the whole target.

### An application is advertised as supported but remains incomplete

Define support through its Application Contract rather than through an enabled
target name. Fail `check` and `apply` when a required capability has no owner.
Show optional gaps without making them failures, and keep contracts limited to
features this configuration actually expects.

### Capability labels hide an unsafe file merge

Require every claim to include its concrete setting/output boundary. Disjoint
labels are insufficient when two contributors replace one monolithic file or
when precedence is undocumented. Use one complete owner in that case.

### Capability modeling becomes another plugin framework

Keep a small typed enum and trusted claims compiled into adapters or emitted by
the local Nix module. No arbitrary runtime claim registration, discovery,
dependency resolution, or third-party capability manifests.

### Stylix updates invalidate audited target claims

Pin Stylix through the flake lock, record the evaluated Stylix revision in the
generated inventory, and verify each enabled target's claimed setting boundary
against inspected or fixture output. A revision change that affects an audited
target requires refreshing that evidence before the claim is trusted.

### Appearance Library names hide their real source

Require integration-specific names such as `stylix-cursors` and record the
public option path, source kind, and cadence in the generated inventory. A
future runtime cursor library receives a different name even if it contains the
same cursor package.

### A Theme Bundle appears applied while declarative parts are stale

Compare the desired bundle with the effective Nix-generated inventory before
runtime writes. Reject stale `rebuild` selections, report `session` selections
as pending, and never imply cross-domain atomicity.

### Nix and runtime bundle definitions drift

Make Nix-visible configuration authoritative for every bundle that references a
rebuild-owned item and emit its runtime representation from Home Manager. Allow
fully runtime-owned bundle definitions only when they contain no declarative
asset references.

### Theme Bundles turn Color Palettes into universal theme objects

Store references in the bundle. Keep colors in Color Palettes, portable
structure in Structure Profiles, and packages/assets in typed Appearance
Libraries. Do not copy their contents into the bundle.

### Noctalia templates become an arbitrary execution escape hatch

Keep the adapter fixed to `noctalia theme`. Headless mode accepts only explicit,
audited template inputs and contained relative outputs; reject hook and dynamic
path fields. Shell-driven hooks are allowed only as a visible delegation to a
selected Noctalia template whose effects were recorded during Phase 0.

### A Noctalia template and built-in adapter both render application colors

Treat this exactly like a Stylix collision: one owner per file and setting.
Split colors from structural effects only at a real application or compositor
include boundary. `theme-manager check` fails on known overlap instead of relying
on last-writer-wins order.

### Live Noctalia and rebuild-time Stylix palettes diverge

Choose a documented authority mode. Do not claim system-wide live consistency
when Stylix targets retain a rebuild-time snapshot.

### Base16 and Material roles do not map cleanly

Keep mappings provider-specific, explicit, minimal, and tested. Fail when a
required role has no mapping instead of guessing silently.

### Multiple palettes recreate a general provider graph

Allow only named provider instances and a default/optional wire override. No
provider chaining, inheritance, fallback graphs, or transformation pipelines.

### Headless Noctalia still brings a large package closure

Treat it as optional. Keep Static and JSON-file providers working. If closure
size becomes a real problem, evaluate extracting or replacing the renderer then;
do not fork it preemptively.

### Watchers miss atomic replacements

Watch parent directories, filter logical filenames, and test rename-based
replacement explicitly.

### Simplified ownership overwrites user files

Use fixed or contained relative filenames beneath one exclusively owned
generated directory for Theme Manager-owned output. Never write directly to a
primary application configuration or a store path. For shell-driven Noctalia
templates that intentionally do so through audited apply hooks, report the
delegation and let Noctalia own apply/undo behavior rather than claiming Theme
Manager transaction guarantees.

### Profiles accumulate unused universal fields

Inventory consumers, remove unused fields, and add new structure only when at
least one real adapter needs it.

### The TUI drives the data model

Finish and test library/CLI operations first. The TUI displays existing concepts
and does not introduce its own state or actions.

## Completion criteria

The migration is complete when:

- the core has no dependency on a particular WM, DE, shell, terminal, or color
  generator;
- Color Providers only produce normalized colors;
- Color Palettes explicitly select their provider and scheme/input;
- Structure Profiles contain only portable appearance intent;
- Theme Bundles explicitly reference their Color Palette, Structure Profile,
  and every selected Appearance Library and item;
- selected cursors, typography, icons, and wallpapers retain explicit source,
  installation/effective state, and delivery cadence, with no package data
  smuggled into palettes;
- the local `stylix-cursors` library demonstrably configures a custom cursor
  package, name, and size through public Stylix options;
- Application Contracts define the required and optional outcomes actually used
  by each configured application;
- one Capability Plan powers `check`, `status`, and `apply`, rejects missing or
  duplicate required ownership, and reports optional gaps and cadence;
- multi-contributor applications are allowed only at verified setting and
  merge/include boundaries;
- Application Wires are typed configuration over built-in Target Adapters,
  including one constrained Noctalia Template Adapter rather than a general
  extension interface;
- Noctalia works as both a Color Provider and an optional template-wire renderer
  in shell-driven and headless modes where configured;
- Stylix is optional, direct, and independently owned;
- every overlap among Stylix, built-in adapters, and Noctalia templates has
  documented disjoint setting ownership;
- Nix activation, runtime-owned files, and delegated Noctalia remain explicit
  execution domains with no false global atomicity claim;
- Niri and Foot are removable configured integrations rather than engine
  assumptions;
- another compositor or custom Quickshell can be added at the adapter/wire edge
  without changing providers, profiles, or the resolver;
- the TUI is optional;
- no general runtime plugin platform remains;
- both hosts pass the verification checklist;
- obsolete code, dependencies, tests, and documentation are removed.

## Implementation record

Phase 0 produced the ownership matrix and golden Niri/Foot output before the
runtime rewrite. The subsequent phases used that evidence to remove conflicts,
replace Luau with typed built-ins, add the constrained Noctalia adapter, add the
Nix-generated Stylix cursor library/inventory, make Niri and Foot includes
optional, and delete obsolete infrastructure. `PHASE-0-INVENTORY.md` records
both the original findings and their final resolution.
