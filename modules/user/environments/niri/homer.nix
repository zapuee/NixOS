{ shared, lib, ... }:

{
  config = lib.mkIf (shared.hostname == "homer") {
    wayland.windowManager.niri.settings = {
      layout = {
        focus-ring = {
          width = 3;
          active-color = "#30303080";
          inactive-color = "#18181860";
          urgent-color = "#3a3a3a80";
        };
      };
    };
  };
}
