{
  config,
  lib,
  pkgs,
  ...
}:

{
  config = lib.mkIf (config.userSettings.terminal == "foot") {
    home.packages = [
      pkgs.foot
      pkgs.nerd-fonts.lilex
    ];

    programs.foot = {
      enable = true;

      settings = {

        csd = {
          preferred = "client";
          size = 0;
        };

        main = {
          font = "Lilex Nerd Font Mono:size=10";
          pad = "5x5";

          include = lib.optional (
            config.userSettings.desktopShell == "noctalia"
          ) "~/.config/foot/themes/noctalia";
        };

        scrollback = {
          lines = 10000;
        };
      };
    };
  };
}
