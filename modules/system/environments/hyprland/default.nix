{ config, lib, pkgs, ... }:

{
  config = lib.mkIf (config.systemSettings.environment == builtins.baseNameOf ./.) {
    systemSettings.bootloader = lib.mkForce "systemd"; # force systemd

    programs.hyprland = {
      enable = true;
      xwayland.enable = true;
      portalPackage = pkgs.xdg-desktop-portal-hyprland;
    };

    services.dbus.enable = true;

    services.pipewire = {
      enable = true;
      alsa.enable = true;
      alsa.support32Bit = true;
      pulse.enable = true;
    };

    security.rtkit.enable = true;
  };
}
