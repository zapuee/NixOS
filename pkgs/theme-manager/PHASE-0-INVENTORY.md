# Phase 0 appearance ownership inventory

This is the evidence baseline for the migration in `PLAN.md`. It describes the
repository at Noctalia revision
`9f35ead435feaa3cc6bcfc6585433abc9ae4db17`. It does not treat mutable settings
inside a running Noctalia Shell as declarative state.

## Scope and host selection

`homer` and `bart` currently select the same relevant applications:

| Area | Effective selection | Source |
|---|---|---|
| Desktop | Niri with Noctalia Shell | host Home Manager modules |
| Runtime structure | Theme Manager, default profile `glass` | `modules/user/theme-manager/config.toml` |
| Runtime colors | Noctalia JSON palette bridge | Theme Manager provider input |
| Terminal | Foot | `userSettings.terminal` |
| Browser | Firefox with Pywalfox and UltimaDark | `userSettings.browser` and Firefox module |
| Prompt | Starship with `useNoctalia = true` | host Home Manager modules |
| Stylix | Not installed or selected | repository search and flake inputs |

The hosts differ in monitor topology, wallpaper-server directory, and the
ignored local Starship style name. Those differences do not currently change a
Theme Manager output.

## Findings that must be resolved before Phase 1

1. **Noctalia template selection is not fully declarative.** The evaluated
   repository declares only the user template `theme-manager-palette`.
   `builtin_ids` and `community_ids` are empty. Foot, Starship, and Pywalfox
   therefore depend on mutable shell settings or may not receive a theme at all.
2. **Foot alpha has two owners.** Home Manager writes
   `[colors-dark].alpha = 0.5`; Theme Manager's included file writes both dark
   and light alpha from the active profile. Include ordering currently decides
   the winner instead of an ownership rule.
3. **Niri focus-ring settings have two owners.** Each host declares width and
   colors in Nix while Theme Manager emits width and colors in its included KDL.
   The effective result depends on Niri's merge/order behavior.
4. **Firefox styling has overlapping participants.** Pywalfox, the Noctalia
   native-messaging host, and the force-installed UltimaDark extension are all
   present. The repository does not prove which browser surfaces each owns.
5. **Noctalia hooks may cross Home Manager boundaries.** Its Foot and Starship
   templates have post/undo hooks that edit application configuration. These
   hooks are unsafe to call composable until behavior against Home
   Manager-managed files is tested.
6. **Live reload is not proven.** Generated files are refreshed atomically, but
   neither the current configuration nor tests prove that every already-running
   application reloads them. Application contracts must distinguish file update
   from live process update.

These are migration blockers, not reasons to add a general provider-combining
layer. The improvement is to make each selection declarative and give each
capability one owner.

## Current ownership and capability matrix

Status meanings:

- **verified**: repository configuration or golden generated output proves it;
- **configured**: Nix declares it, but effective visual behavior was not tested;
- **unresolved**: it depends on mutable state or an unproved merge boundary;
- **unused**: the profile resolves it, but no configured adapter renders it.

### Color pipeline

| Capability | Requirement | Owner | Explicit source | Cadence | Boundary | Status |
|---|---|---|---|---|---|---|
| Semantic palette export | required | Noctalia user template | `theme-manager-palette` | runtime, shell-driven | `$XDG_CACHE_HOME/theme-manager/noctalia-palette.json` | verified configuration |
| Palette normalization | required | Theme Manager Noctalia provider | `luau:noctalia` | runtime | provider reads the JSON cache | verified by integration test |
| Palette tokens | required by current profiles/adapters | Noctalia | `primary`, `outline`, `error`, `shadow`, `secondary`, `surface`, `on_surface`, plus exported Material tokens | runtime | JSON fields | configured; only seven are required today |
| Static fallback palette | development/test only | Theme Manager | `luau:static` settings | invocation | in-memory palette | verified by CLI tests |

The palette exporter currently exists only when `desktopShell = "noctalia"`.
That coupling must be removed before the headless Noctalia-provider claim in the
plan is complete.

### Niri

| Typed capability | Requirement | Owner | Source | Cadence | Concrete boundary | Status |
|---|---|---|---|---|---|---|
| layout gaps | required | Theme Manager Niri adapter | `layout.gaps` | runtime | generated `layout.gaps` | verified golden |
| outer gaps | required | Theme Manager Niri adapter | `layout.outer_gaps - layout.gaps` | runtime | four generated `struts` | verified golden |
| focus-ring width and colors | required | Theme Manager and host Nix | profile plus host `niri.nix` | mixed | same Niri settings | unresolved conflict |
| shadow color/strength | required | Theme Manager Niri adapter | `shadow.*` | runtime | generated `layout.shadow` | verified golden |
| global corner radius | required | Theme Manager Niri adapter | `roles.compositor.corner_radius` | runtime | global `window-rule` | verified golden |
| border background drawing | required current behavior | Theme Manager Niri adapter | adapter constant | runtime | global `window-rule` | verified golden |
| browser corner radius | required | Theme Manager Niri adapter | `roles.browser.corner_radius` | runtime | Firefox app-id rule | verified golden |
| browser opacity | required | Theme Manager Niri adapter | `roles.browser.background_opacity` | runtime | Firefox app-id rule | verified golden |
| browser blur | optional | Theme Manager Niri adapter | `roles.browser.blur` | runtime | Firefox app-id rule | verified golden |
| terminal corner radius | required | Theme Manager Niri adapter | `roles.terminal.corner_radius` | runtime | Foot app-id rule | verified golden |
| terminal blur | optional | Theme Manager Niri adapter | `roles.terminal.blur` | runtime | Foot app-id rule | verified golden |
| terminal window opacity | explicitly excluded | none | `use_opacity = false` | runtime | Foot app-id rule | verified absent |
| Picture-in-Picture exception | required current behavior | local Nix | Niri module | rebuild/session | dedicated Firefox title rule | configured; merge order needs a test |
| monitor topology | host-specific, outside theme contract | local Nix | `hosts/homer/niri.nix` | rebuild/session | Niri output blocks | configured |

### Foot

| Typed capability | Requirement | Owner | Source | Cadence | Concrete boundary | Status |
|---|---|---|---|---|---|---|
| terminal colors | required | intended Noctalia built-in template | Noctalia `foot` template | runtime, shell-driven | `~/.config/foot/themes/noctalia` | unresolved; template ID is not declared |
| background alpha | required | Theme Manager and local Nix | terminal role / literal `0.5` | mixed | `[colors-dark].alpha`; Theme Manager also owns light alpha | unresolved conflict |
| font family and size | required | local Nix | `pkgs.nerd-fonts.lilex`, `Lilex Nerd Font Mono:size=10` | rebuild/session | Foot `main.font` | verified configuration |
| padding | optional | local Nix | `5x5` | rebuild/session | Foot `main.pad` | configured |
| CSD | optional | local Nix | client, size zero | rebuild/session | Foot `csd` | configured |
| scrollback | optional | local Nix | 10000 lines | rebuild/session | Foot `scrollback.lines` | configured |
| cursor and selection colors | optional current feature | intended Noctalia template | Noctalia Foot palette fields | runtime | Noctalia Foot theme file | unresolved |
| live reload | optional until proven | none | none | runtime | running Foot process | unverified |

The Theme Manager fragment is appended after the Noctalia include by the Home
Manager module. This is a real file-include boundary, but it is composable only
for disjoint keys. It does not justify two alpha owners.

### Firefox

| Typed capability | Requirement | Owner | Source | Cadence | Concrete boundary | Status |
|---|---|---|---|---|---|---|
| browser palette | required by intended design | intended Noctalia/Pywalfox template | likely `pywalfox-beta4+` or legacy equivalent | runtime, shell-driven | Pywalfox color cache/message | unresolved; no community ID declared |
| native messaging host | required for Noctalia/Pywalfox | local activation invoking pinned Noctalia | `noctalia firefox-theme install` | activation | `~/.mozilla/native-messaging-hosts/pywalfox.json` | configured |
| browser chrome theme | optional | UltimaDark extension | forced AMO install | extension lifecycle | Firefox extension state | configured; overlap untested |
| transparent browser support | required for glass intent | local Nix | Firefox preferences | rebuild/session | profile preferences | configured |
| Niri window structure | required | Theme Manager Niri adapter | browser role | runtime | Niri Firefox app-id rule | verified golden |

The install command configures the native-messaging host; it does not itself
select the Noctalia community color template. No Firefox merge boundary is
accepted until Pywalfox and UltimaDark ownership is visually and mechanically
tested.

### Starship

| Typed capability | Requirement | Owner | Source | Cadence | Concrete boundary | Status |
|---|---|---|---|---|---|---|
| prompt palette | required by current `useNoctalia` choice | intended Noctalia built-in template | Noctalia `starship` template | runtime, shell-driven | cache palette plus edits to Starship config | unresolved; built-in ID is not declared |
| prompt layout/modules | required | Starship defaults or mutable config | local style is disabled while `useNoctalia = true` | mixed | Starship config | unresolved |
| local `grayscale`/`backrooms` style | unused | none | host option | rebuild | no settings emitted | verified unused |

The Noctalia Starship hook edits the selected Starship configuration and its
undo hook removes the inserted palette block. This is not yet a proven safe
boundary for a Home Manager-owned file.

### Cursor, typography, icons, and wallpaper

| Typed capability | Current owner and value | Package/source | Cadence | Planned explicit library | Status |
|---|---|---|---|---|---|
| terminal typography | local Nix: Lilex Nerd Font Mono 10 | `pkgs.nerd-fonts.lilex` | rebuild/session | future local or `stylix-fonts` item must name this explicitly | configured |
| general system fonts | local Nix: Noto, Noto CJK Sans, Noto Color Emoji | Nixpkgs packages | rebuild | future local or `stylix-fonts` items | configured |
| cursor | no repository selection found | none | n/a | no `stylix-cursors` item selected | absent |
| icon theme | no repository selection found | none | n/a | no `stylix-icons` item selected | absent |
| wallpaper appearance | mutable Noctalia state, if any | not declared | runtime | no `stylix-wallpapers` item selected | unresolved |
| wallpaper file server | local Nix service | host Pictures directory | session | outside appearance ownership | configured |

There are no selected Stylix targets and no Stylix appearance libraries in the
current repository. Future cursor choices must use an explicit qualified form
such as `library = "stylix-cursors"`, never an unqualified name.

## Structure-profile field consumption

| Profile field | Current consumer | Status |
|---|---|---|
| `layout.gaps`, `layout.outer_gaps` | Niri adapter | used |
| all `focus_ring.*` fields | Niri adapter | used |
| `shadow.strength`, `shadow.color` | Niri adapter | used |
| `roles.compositor.corner_radius` | Niri adapter | used |
| `roles.browser.background_opacity`, `blur`, `corner_radius` | Niri adapter | used |
| `roles.terminal.background_opacity` | Foot adapter | used |
| `roles.terminal.blur`, `corner_radius` | Niri adapter | used |
| `variables.material`, every role `material` | none | unused |
| `variables.opacity.tertiary` | none | unused |
| `variables.radius.secondary` | none | unused |
| `roles.browser.accent_color` | none after resolution | unused output |
| `roles.terminal.cursor_color`, `selection_color` | none after resolution | unused output |
| palette `surface`, `on_surface` | no configured adapter | unused |

The resolver still resolves role color references even when no target writes
them. Until the resolver is simplified, `primary` and `secondary` remain input
requirements because of the unused accent/cursor/selection fields.

## Initial application contracts

These are baseline contracts, not the final schema.

### Foot contract

Required capabilities are `colors`, `font`, and `background_alpha`. Optional
capabilities are `cursor_color`, `selection_color`, `padding`, `csd`,
`scrollback`, and `live_reload`. A valid plan must assign exactly one owner to
alpha. The Noctalia theme include and Theme Manager fragment may coexist only
after the Noctalia template ID is declared and their keys are proven disjoint.

### Firefox contract

Required capabilities are `colors`, `native_messaging` when Pywalfox is the
color wire, and the current transparent-surface preferences for the glass
profile. Niri opacity/blur/radius are compositor capabilities, not Firefox file
capabilities. UltimaDark is optional and must either own a disjoint browser
surface or be removed.

### Niri contract

Required capabilities are layout gaps, focus ring, shadow, global radius, and
the configured browser/terminal window rules. Host output topology and the
Picture-in-Picture exception remain local Nix policy. The generated include is
a valid boundary only once duplicate focus-ring ownership is removed.

### Starship contract

Required capabilities are prompt layout and colors. The repository currently
cannot say which source supplies either under `useNoctalia`; explicit template
selection and a non-destructive config boundary are required first.

## Noctalia template inventory

| Template | Selection | Output | Hooks | Shell dependency | Headless-wire eligibility |
|---|---|---|---|---|---|
| user `theme-manager-palette` | declared | Theme Manager JSON cache | none declared | currently yes, due to module placement | yes after decoupling/export support |
| built-in Foot | not declared | `~/.config/foot/themes/noctalia` | edits/removes Foot include and theme file | current intended path is shell-driven | renderer is eligible; hooks need replacement |
| built-in Starship | not declared | Noctalia Starship palette cache and Starship config block | edits/removes palette selection/block | current intended path is shell-driven | renderer is eligible; config mutation is not |
| community Pywalfox | not declared or pinned | Pywalfox color cache | invokes Firefox theme update | current intended path is shell-driven | only after exact template/revision is pinned |

## Golden-output baseline

The fixtures under
`crates/theme-manager/tests/fixtures/v0.2/{glass,paper}` pin the exact Niri and
Foot output produced from the packaged static palette. The CLI integration test
applies both profiles through the real engine and compares every byte. This
protects current behavior while provider and adapter names change.

The fixture intentionally excludes the state database because it contains
environment-specific paths and timestamps.

## Performance baseline and constraints

The current outputs are tiny and apply is primarily filesystem work. No
performance rewrite is justified in Phase 0. Existing useful behavior to retain:

- render and preflight every target before writing any output;
- skip unchanged outputs;
- load and normalize the active palette once per apply;
- keep planning linear in the number of capabilities and wires;
- watch only declared runtime inputs after the general plugin discovery system
  is removed.

Benchmarks should be added only if later profiling reveals a meaningful hot
path. Correct ownership and deterministic output are the stability work now.

## Gate for the next iteration

Before Phase 1 changes runtime behavior:

1. declare the intended Noctalia Foot, Starship, and Firefox template IDs, or
   explicitly remove those integrations;
2. choose one Foot alpha owner and one Niri focus-ring owner;
3. decide whether UltimaDark and Pywalfox own demonstrably disjoint Firefox
   surfaces;
4. decide the initial palette authority when Noctalia Shell is absent; and
5. keep Stylix disabled unless an explicit target or qualified appearance
   library item is selected.

## Final resolution

The v0.3 migration resolved every gate above:

- Noctalia now declaratively selects the built-in Starship template and
  repository-owned `foot-colors`, `pywalfox`, and normalized palette templates.
  No community template is fetched at runtime. Foot and palette templates have
  no hooks; Pywalfox retains only the audited `firefox-theme` action.
- Theme Manager is the only Foot alpha owner. Noctalia owns Foot colors,
  cursor color, and selection color through a separate include.
- Theme Manager is the only Niri focus-ring owner. The local
  Picture-in-Picture policy is emitted after Theme Manager's general include so
  its zero-radius exception wins deterministically.
- UltimaDark was removed. The pinned repository Pywalfox template is the sole
  Firefox color owner, while local Nix owns native messaging and browser
  preferences and Niri owns window structure.
- Starship structure remains declarative. Its Noctalia palette is written to a
  mutable cache path selected by `STARSHIP_CONFIG`, avoiding edits to an
  immutable Home Manager file.
- Shell JSON is the current hosts' explicit palette authority. The same runtime
  also supports Static, normalized JSON, and shell-independent Noctalia image
  or theme-JSON inputs.
- Stylix is now selected only for the qualified
  `stylix-cursors/bibata-modern-ice` item. Nix pins the package, name, size,
  upstream revision, GTK/X11 cursor subtargets, and consumer coverage in the
  generated inventory. Stylix application/color targets remain disabled.
- The Capability Plan combines these local Nix, Stylix, built-in, and Noctalia
  claims. Missing coverage, duplicate ownership, unsafe output collisions, and
  stale declarative selections fail before runtime writes.
- Unconsumed profile fields (`material`, tertiary opacity, secondary radius,
  browser accent, and terminal cursor/selection roles) were deleted. A
  `surface_style` field was not added because no adapter currently consumes it.

Golden Niri/Foot fixtures still match byte for byte. The full flake check,
release package checks, Rust tests, and both host evaluations pass with this
ownership model.
