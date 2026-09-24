{ config, lib, pkgs, ... }:

let
  cfg = config.userSettings.theme-manager;

  themeManager =
    pkgs.callPackage
      ../../../../pkgs/theme-manager/package.nix
      { };

  themeManagerDir =
    "${config.home.homeDirectory}/NixOS/pkgs/theme-manager";
in
{
  options.userSettings.theme-manager.enable =
    lib.mkEnableOption "Theme Manager";

  config = lib.mkIf cfg.enable {
    home.packages = [
      themeManager
    ];

    xdg.configFile."theme-manager/config.toml" = {
      source =
        config.lib.file.mkOutOfStoreSymlink
          "${themeManagerDir}/examples/config.toml";

      force = true;
    };

    xdg.configFile."theme-manager/profiles/glass.toml" = {
      source =
        config.lib.file.mkOutOfStoreSymlink
          "${themeManagerDir}/examples/profiles/glass.toml";

      force = true;
    };

    xdg.configFile."theme-manager/profiles/paper.toml" = {
      source =
        config.lib.file.mkOutOfStoreSymlink
          "${themeManagerDir}/examples/profiles/paper.toml";

      force = true;
    };

    xdg.configFile."theme-manager/integrations/noctalia/palette.template.json" = {
      source =
        config.lib.file.mkOutOfStoreSymlink
          "${themeManagerDir}/integrations/noctalia/palette.template.json";

      force = true;
    };

    systemd.user.services.theme-manager = {
      Unit = {
        Description = "Theme Manager";
        After = [ "graphical-session.target" ];
        PartOf = [ "graphical-session.target" ];
      };

      Service = {
        ExecStart = "${themeManager}/bin/theme-manager watch";
        Restart = "on-failure";
        RestartSec = 1;
      };

      Install.WantedBy = [
        "graphical-session.target"
      ];
    };
  };
}
