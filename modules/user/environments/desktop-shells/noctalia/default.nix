{ config, inputs, lib, ... }:

{
  imports = [
    inputs.noctalia.homeModules.default
  ];

  config = lib.mkIf (config.userSettings.desktop-shell == "noctalia") {
    programs.noctalia = {
      enable = true;

      settings.plugins.enabled = [
        "zap/theme-manager"
      ];
    };

    xdg.dataFile."noctalia/plugins/theme-manager".source =
      config.lib.file.mkOutOfStoreSymlink
        "${config.home.homeDirectory}/NixOS/modules/user/environments/desktop-shells/noctalia/plugins/theme-manager";
  };
}
