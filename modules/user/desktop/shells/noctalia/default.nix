{
  config,
  inputs,
  lib,
  ...
}:
let
  themeConfig = builtins.fromTOML (builtins.readFile ../../../theme-manager/config.toml);
  shellTemplates = map (wire: wire.template) (
    builtins.attrValues (
      lib.filterAttrs (
        _name: wire:
        wire.adapter == "noctalia-template" && wire.execution == "shell" && (wire.enabled or true)
      ) themeConfig.application_wires
    )
  );
  builtinIds = map (lib.removePrefix "builtin:") (
    lib.filter (template: lib.hasPrefix "builtin:" template) shellTemplates
  );
  userTemplateCatalog = {
    "user:foot-colors" = {
      input_path = "$XDG_CONFIG_HOME/theme-manager/noctalia/foot.ini";
      output_path = "$XDG_CONFIG_HOME/foot/themes/noctalia";
    };
    "user:pywalfox" = {
      input_path = "$XDG_CONFIG_HOME/theme-manager/noctalia/pywalfox.json";
      output_path = "$XDG_CACHE_HOME/wal/colors.json";
      post_action = "firefox-theme";
    };
  };
  selectedUserTemplates = lib.filter (template: lib.hasPrefix "user:" template) shellTemplates;
  userTemplates = builtins.listToAttrs (
    map (
      template: lib.nameValuePair (lib.removePrefix "user:" template) userTemplateCatalog.${template}
    ) selectedUserTemplates
  );
  usesShellPalette = lib.any (
    palette:
    themeConfig.color_providers.${palette.provider}.kind == "noctalia" && palette.source == "shell-json"
  ) (builtins.attrValues themeConfig.color_palettes);
in
{
  imports = [
    inputs.noctalia.homeModules.default
  ];

  config = lib.mkIf (config.userSettings.desktopShell == "noctalia") {
    programs.noctalia = {
      enable = true;

      settings = {
        theme.templates = {
          builtin_ids = builtinIds;
          community_ids = [ ];

          user =
            userTemplates
            // lib.optionalAttrs usesShellPalette {
              theme-manager-palette = {
                input_path = "$XDG_CONFIG_HOME/theme-manager/noctalia/palette.json";
                output_path = "$XDG_CACHE_HOME/theme-manager/noctalia-palette.json";
              };
            };
        };
      };
    };

    xdg.configFile = {
      "theme-manager/noctalia/palette.json".source =
        ../../../../../pkgs/theme-manager/integrations/noctalia/palette.json;
      "theme-manager/noctalia/foot.ini".source =
        ../../../../../pkgs/theme-manager/integrations/noctalia/foot.ini;
      "theme-manager/noctalia/pywalfox.json".source =
        ../../../../../pkgs/theme-manager/integrations/noctalia/pywalfox.json;
    };
  };
}
