{ config, inputs, lib, ... }:

let
  noctaliaDir =
    "${config.home.homeDirectory}/NixOS/modules/user/environments/desktop-shells/noctalia";
in
{
  imports = [
    inputs.noctalia.homeModules.default
  ];

  config = lib.mkIf (config.userSettings.desktop-shell == "noctalia") {
    programs.noctalia = {
      enable = true;

      settings = {
        plugins.enabled = [
          "zap/theme-manager"
        ];

        theme.templates.user."theme-manager-niri" = {
          input_path =
            "$XDG_CONFIG_HOME/theme-manager/niri-colors.template.kdl";

          output_path =
            "$XDG_CONFIG_HOME/theme-manager/niri-colors.kdl";
        };
      };
    };

    # Plugin itself
    xdg.dataFile."noctalia/plugins/theme-manager".source =
      config.lib.file.mkOutOfStoreSymlink
        "${noctaliaDir}/plugins/theme-manager";

    # Profiles the plugin can read
    xdg.dataFile."noctalia/theme-manager/noctalia-profiles".source =
      config.lib.file.mkOutOfStoreSymlink
        "${noctaliaDir}/noctalia-profiles";

    xdg.dataFile."noctalia/theme-manager/structure-profiles".source =
      config.lib.file.mkOutOfStoreSymlink
        "${noctaliaDir}/structure-profiles";
  };
}
