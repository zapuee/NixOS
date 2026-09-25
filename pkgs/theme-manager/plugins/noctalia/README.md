# Noctalia provider plugin

Noctalia is an optional Luau color provider. Nothing in the Rust application
depends on it.

## Palette bridge

Register `palette.template.json` as a Noctalia user template and render it to:

```text
$XDG_CACHE_HOME/theme-manager/noctalia-palette.json
```

Packaged installations place the template at
`$THEME_MANAGER_DATA_DIR/plugins/noctalia/palette.template.json`.

Select this provider in `config.toml`:

```toml
[provider]
kind = "luau:noctalia"

[provider.inputs]
palette = "$XDG_CACHE_HOME/theme-manager/noctalia-palette.json"
```

Noctalia rerenders user templates whenever its palette changes.
`theme-manager watch` watches the rendered JSON and reapplies enabled targets.

To force the first render:

```bash
noctalia msg templates-apply
```
