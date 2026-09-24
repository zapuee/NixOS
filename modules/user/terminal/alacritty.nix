{
  config,
  lib,
  pkgs,
  ...
}:

{
  config = lib.mkIf (config.userSettings.terminal == "alacritty") {
    home.packages = [ pkgs.alacritty ];
    programs.alacritty.enable = true;
    programs.alacritty.settings = {
      window.opacity = lib.mkForce 0.85;
    };
  };
}
