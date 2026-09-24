{ config, inputs, lib, ... }:

{
  imports = [
    inputs.noctalia.homeModules.default
  ];

  config = lib.mkIf (config.userSettings.desktop-shell == "noctalia") {
    programs.noctalia = {
      enable = true;

      settings = {
        theme.templates.user."theme-manager-palette" = {
          input_path =
            "$XDG_CONFIG_HOME/theme-manager/integrations/noctalia/palette.template.json";

          output_path =
            "$XDG_CACHE_HOME/theme-manager/noctalia-palette.json";
        };
      };
    };
  };
}
