{
  lib,
  osConfig,
  ...
}:

{
  config = lib.mkIf (osConfig.systemSettings.desktop == builtins.baseNameOf ./.) {
    programs.waybar = {
      enable = true;
      systemd.enable = true;

      settings.mainBar = {
        layer = "top";
        position = "top";

        modules-left = [ "hyprland/workspaces" ];
        modules-center = [ "clock" ];
        modules-right = [
          "pulseaudio"
          "network"
          "battery"
        ];

        clock.format = "{:%H:%M}";
        battery.format = "{capacity}%";

        network = {
          format-wifi = "WiFi {essid}";
          format-ethernet = "Ethernet";
          format-disconnected = "Disconnected";
        };

        pulseaudio.format = "Vol {volume}%";
      };
    };
  };
}
