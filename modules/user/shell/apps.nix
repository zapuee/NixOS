{ config, lib, pkgs, ... }:

let
  cfg = config.userSettings.shell.apps;
in
{
  options = {
    userSettings.shell.apps = {
      enable = lib.mkEnableOption "Enable a collection of additional useful CLI apps";
    };
  };

  config = lib.mkIf cfg.enable {
    # Collection of useful CLI apps
    home.packages = with pkgs; [
      # Command Line
      killall
      trashy
    ];

  };
}
