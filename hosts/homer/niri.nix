{ lib, osConfig, ... }:

{
  config = lib.mkIf (osConfig.systemSettings.desktop == "niri") {
    wayland.windowManager.niri = {
      settings = {
        layout = {
          focus-ring = {
            width = 2;
            active-color = "#00000080";
            inactive-color = "#00000000";
            urgent-color = "#00000000";
          };

          border.off = { };
        };
      };

      extraConfig = ''
        output "DP-3" {
          mode "2560x1080@143.945"
          position x=0 y=0
        }

        output "DP-4" {
          mode "2560x1080@74.991"
          position x=-2560 y=0
        }
      '';
    };
  };
}
