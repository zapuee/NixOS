{ lib, osConfig, ... }:

{
  config = lib.mkIf (osConfig.systemSettings.desktop == "niri") {
    wayland.windowManager.niri.settings = {
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
  };
}
