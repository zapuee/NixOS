# Architecture notes

## Stable contracts

### Profile contract

A profile expresses semantic appearance. It must not name concrete applications.

### Provider contract

```rust
pub trait PaletteProvider: Send + Sync {
    fn id(&self) -> &str;
    fn load(&self) -> Result<Palette>;
    fn watch_paths(&self) -> Vec<PathBuf>;
}
```

Providers own dynamic inputs only. They do not generate application files.

### Target contract

```rust
pub trait Target: Send + Sync {
    fn id(&self) -> &str;
    fn output_paths(&self) -> Result<Vec<PathBuf>>;
    fn render(&self, theme: &ResolvedTheme) -> Result<Vec<Artifact>>;
}
```

Targets are pure renderers in v0.1. The core performs compare-before-write and
atomic replacement, keeping application adapters small and consistent.
Declaring output paths lets runtime component controls remove only generated
files owned by a disabled target. Output ownership uses canonical paths and
content hashes recorded in state, so format-neutral outputs are removed only
when they still match what Rust last wrote. Collision checks resolve lexical and
symlink aliases, preventing two adapters from targeting the same location.

### Runtime component state

`config.toml` is the declarative boundary: a target with `enabled = false`
cannot be enabled by runtime controls. For configured targets, the state file
stores a set of runtime-disabled target IDs. State updates and target applies
share a file lock, and state writes use atomic replacement. This keeps the CLI,
TUI, and watcher from losing fields or recreating a target during a concurrent
disable.

## Runtime extension boundary

Native Rust ABI is not stable. Runtime-loaded `.so` adapters would couple every
plugin to the exact compiler/build ABI and make upgrades fragile.

Theme-Manager therefore embeds a sandboxed Luau host through a versioned,
plain-table API. Luau code only converts provider inputs into palette roles or a
resolved theme into named text artifacts. Rust owns path expansion, input
reads, output writes, collision detection, content ownership, state locking,
watching, validation, and resource limits. No provider or target is registered
in compiled Rust; the registry starts empty and is populated only by discovered
plugin manifests. The official Static, Noctalia, Niri, and Foot integrations
use exactly the same boundary as third-party plugins.

Every configured target has an instance ID separate from its adapter ID. For
example, `desktop-primary` may use `luau:generic-text`; runtime toggles and state
refer to `desktop-primary`. Instance IDs remain stable even when their adapter
implementation changes.

Plugins cannot choose filesystem paths or perform writes. Configuration maps
their logical output names to paths, and provider inputs are read by Rust before
the plugin runs. A fresh, memory-limited VM is created for each call and an
execution deadline interrupts runaway code. This keeps Rust responsible for the
heavy work and makes integrations terminal- and application-agnostic.

## Why the Noctalia palette bridge is a file

Noctalia v5.1 can automatically rerender user templates when its palette changes.
A rendered JSON file therefore gives us a narrow, stable integration boundary:

```text
Noctalia palette -> palette.template.json -> cache JSON -> provider -> engine
```

The core does not call Noctalia APIs and can run without Noctalia installed.
