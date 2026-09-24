{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.userSettings.shell.fun;
in
{
  options = {
    userSettings.shell.fun = {
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
      tty-clock
    ];
  };
}
