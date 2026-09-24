# Noctalia integration

Noctalia is an optional Luau color-provider plugin, not the Theme-Manager
runtime or UI. The plugin is packaged at
`$THEME_MANAGER_DATA_DIR/plugins/noctalia`; it is not linked into the Rust
application.

## Palette bridge

The stable bridge is a normal Noctalia user template. Register
`palette.template.json` as a user template and render it to:

```text
$XDG_CACHE_HOME/theme-manager/noctalia-palette.json
```

The template is installed at
`$THEME_MANAGER_DATA_DIR/noctalia/palette.template.json`. When referring to it
from Nix, use the package path rather than a checkout path:

```nix
programs.noctalia.settings.theme.templates.user."theme-manager-palette" = {
  input_path =
    "${themeManagerPackage}/share/theme-manager/noctalia/palette.template.json";
  output_path =
    "$XDG_CACHE_HOME/theme-manager/noctalia-palette.json";
};
```

Noctalia rerenders user templates whenever its palette changes. `theme-manager watch` watches the rendered JSON, resolves the active structure profile again, and reapplies targets.

Select the plugin with:

```toml
[provider]
kind = "luau:noctalia"

[provider.inputs]
palette = "$XDG_CACHE_HOME/theme-manager/noctalia-palette.json"
```

To force the first render:

```bash
noctalia msg templates-apply
```

Use `theme-manager tui` for interactive profile and component controls.
