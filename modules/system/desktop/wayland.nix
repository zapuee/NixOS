{ config, lib, ... }:

let
  cfg = config.systemSettings.desktop;
in
{
  config =
    lib.mkIf
      (builtins.elem cfg [
        "hyprland"
        "niri"
      ])
      {
        services.dbus.enable = true;

        services.pipewire = {
          enable = true;
          alsa.enable = true;
          alsa.support32Bit = true;
          pulse.enable = true;
          wireplumber.enable = true;
        };

        security.rtkit.enable = true;
      };
}
