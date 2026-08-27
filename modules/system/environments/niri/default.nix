{ config, lib, pkgs, ... }:

{
  config = lib.mkIf (config.systemSettings.environment == builtins.baseNameOf ./.) {
    systemSettings.bootloader = lib.mkForce "systemd"; # force systemd

    programs.niri = {
      enable = true;
   };

   # xdg.portal = {
   #   enable = true;
   #   extraPortals = [
   #     pkgs.xdg-desktop-portal-gtk
   #   ];
   # };

    services.dbus.enable = true;

    services.pipewire = {
      enable = true;
      alsa.enable = true;
      alsa.support32Bit = true;
      pulse.enable = true;
      wireplumber.enable = true;
    };

    services.xserver.enable = true;
    programs.xwayland.enable = true;
    security.rtkit.enable = true;
  };
}
