{ pkgs, lib, config, osConfig, ... }:

{
  config = lib.mkIf (osConfig.systemSettings.environment == "niri") {
    home.packages = with pkgs; [
      btop
      nautilus
      xwayland-satellite
      gnome-keyring
    ];

    xdg.portal = {
      enable = true;

      config = {
        common = {
          default = [
            "gnome"
            "gtk"
          ];
        };

        niri = {
          default = [
            "gnome"
            "gtk"
          ];
        };
      };

      extraPortals = with pkgs; [
        xdg-desktop-portal-gnome
        xdg-desktop-portal-gtk
      ];
    };
  };
}
