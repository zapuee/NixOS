# Architecture

The reusable `theme-core` crate contains only portable Theme Bundle and
Application Contract types, profiles, semantic palette validation, resolution,
generic appearance inventory types, capability planning, XDG paths, atomic file
helpers, and minimal state. It contains no named compositor, shell, terminal,
color generator, provider, wire, or adapter type.

The `theme-manager` binary contains four fixed implementation edges:

- `config.rs`: provider- and adapter-specific typed runtime configuration;
- `providers.rs`: Static, JSON-file, and Noctalia color loading;
- `adapters.rs`: Niri and Foot rendering plus audited shell-template claims;
- `runtime.rs`: bundle resolution, one-load-per-palette caching, capability
  validation, staged rendering, contained output installation, and delegated
  Noctalia coordination.

```text
Theme Bundle
  |-- Color Palette -> Color Provider -> normalized Palette
  |-- Structure Profile ----------------------|
  |                                           v
  |                                  Resolved Appearance
  |                                           |
  +-> enabled Application Wires -> Capability Plan -> built-in adapters
  |
  +-> qualified Appearance Library items -> Nix inventory
```

The Capability Plan is validation data, not a composite renderer. A capability
has exactly one owner. Multiple contributors can describe one application only
through distinct audited setting boundaries. Runtime claims are compiled into
adapters; rebuild claims come from the read-only Nix inventory. TOML cannot add
arbitrary claims.

The host's qualified cursor selection is Nix-visible. The same TOML bundle
reference selects a repository-declared item from `stylix-cursors`; Nix resolves
its package/name/size through upstream Stylix and emits the effective inventory.
Theme Manager reads that inventory but never evaluates packages or invokes a
rebuild.

Theme Manager owns only `generated_dir`. Normal adapters have fixed filenames.
A headless Noctalia wire stages output and accepts only a contained relative
destination. Its template is selected from a closed, embedded repository
catalog rather than a path. The Home Manager Noctalia module derives its shell
template selection from the same configured wires. Shell-driven Noctalia and
Nix activation remain separate execution domains; neither is presented as part
of a global atomic transaction.

After all outputs render and validate, changed files are atomically replaced
and any file no longer in the desired set is removed from the exclusively owned
generated directory. Provider or render failure therefore preserves the entire
last-valid set.

The CLI is the sole frontend. The watcher calls the same runtime operations.
No registry, discovery mechanism, scripting VM, target factory, component
toggle database, or TUI-specific data model remains.
