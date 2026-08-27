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
    home.packages = [ pkgs.foot ];

    programs.foot = {
      enable = true;

      settings = {
        main = {
          font = "monospace:size=10";
          pad = "5x5";
        };

        colors = {
          alpha = "0.8";
        };

        scrollback = {
          lines = 10000;
        };
      };
    };
  };
}
