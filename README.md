# NixOS configuration

This flake manages the `bart` and `homer` NixOS hosts, their Home Manager
profiles, and the local `theme-manager` package.

## Layout

```text
flake/                 flake-parts modules and host construction
hosts/<name>/          machine metadata, hardware, system, and home overrides
modules/system/        reusable NixOS feature modules
modules/user/          reusable Home Manager feature modules
pkgs/theme-manager/    standalone Rust package and Home Manager module
```

System features live under `systemSettings`; Home Manager features live under
`userSettings`. A module owns its option declaration and gates its configuration
with that option. Desktop, editor, browser, terminal, Discord-client, Starship,
and desktop-shell choices are discovered from their module or theme files, so a
new implementation does not require updating a separate enum.

## Add a host

Create `hosts/<name>/` with these files:

- `host.nix` — `username`, `stateVersion`, and optional `system` metadata.
- `hardware-configuration.nix` — generated hardware settings.
- `configuration.nix` — host-specific `systemSettings` and overrides.
- `home.nix` — host-specific `userSettings` and imports.

The flake discovers host directories automatically. Shared behavior belongs in
`modules/`; keep only hardware and genuine machine differences under `hosts/`.

## Validate

Use `path:.` while files are still untracked so Nix evaluates the entire working
tree:

```bash
nix fmt -- --ci
nix flake check path:. --no-build
nix flake check path:.
```

The full check runs formatting, Deadnix, and Statix validation, then builds
`theme-manager` and runs its install checks. Its Rust test suite can also be run
directly:

```bash
cargo test --manifest-path pkgs/theme-manager/Cargo.toml --all-targets
```
