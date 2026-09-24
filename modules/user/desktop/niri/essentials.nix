{
  pkgs,
  lib,
  osConfig,
  ...
}:

{
  config = lib.mkIf (osConfig.systemSettings.desktop == "niri") {
    home.packages = with pkgs; [
      wl-clipboard
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
