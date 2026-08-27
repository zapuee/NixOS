{ config, lib, pkgs, ... }:

let
  cfg = config.userSettings.shell.extraApps;
in 
{
  options = {
    userSettings.shell.extraApps = {
      enable = lib.mkEnableOption "Add some fun but mostly useless CLI apps";
    };
  };

  config = lib.mkIf cfg.enable {
    # Fun CLI apps that aren't necessary
    home.packages = with pkgs; [
      fastfetch
      unimatrix
      cava
      lavat
    ];
  };
}
