{ config, lib, pkgs, ... }:

let
  cfg = config.userSettings.foot;
in {
  options = {
    userSettings.foot = {
      enable = lib.mkEnableOption "Enable foot";
    };
  };

  config = lib.mkIf cfg.enable {
    home.packages = [ 
      pkgs.foot
      pkgs.nerd-fonts.lilex
    ];

    programs.foot = {
      enable = true;

      settings = {
        csd = {
          preferred = "client";
          size = 0; # remove title bar
        };

        main = {
          font = "Lilex Nerd Font Mono:size=10";
          pad = "5x5";
        };

        colors = {
          alpha = "0.65";
        };

        scrollback = {
          lines = 10000;
        };
      };
    };
  };
}
