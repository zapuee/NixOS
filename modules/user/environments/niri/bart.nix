{ shared, lib, ... }:

{
  config = lib.mkIf (shared.hostname == "bart") {
    wayland.windowManager.niri.settings.outputs = {
      layout = {
        focus-ring = {
          width = 3;
          active-color = "#c9b45880";
          inactive-color = "#5c563060";
          urgent-color = "#8f7d3280";
        };
      };
    };
  };
}
