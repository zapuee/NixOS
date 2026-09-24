{
  config,
  lib,
  pkgs,
  ...
}:

{
  config = lib.mkIf (config.systemSettings.desktop == builtins.baseNameOf ./.) {
    programs.hyprland = {
      enable = true;
      xwayland.enable = true;
      portalPackage = pkgs.xdg-desktop-portal-hyprland;
    };

  };
}
