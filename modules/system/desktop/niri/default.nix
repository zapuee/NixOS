{ config, lib, ... }:

{
  config = lib.mkIf (config.systemSettings.desktop == builtins.baseNameOf ./.) {
    programs.niri = {
      enable = true;
    };

    # xdg.portal = {
    #   enable = true;
    #   extraPortals = [
    #     pkgs.xdg-desktop-portal-gtk
    #   ];
    # };

    services.xserver.enable = true;
    programs.xwayland.enable = true;
  };
}
