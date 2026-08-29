{ shared, lib, ... }:

{
  config = lib.mkIf (shared.hostname == "homer") {
    wayland.windowManager.niri.settings.outputs = {
      "DP-3" = {
        position = {
          x = 0;
          y = 0;
        };
      };

      "DP-4" = {
        position = {
          x = 2560;
          y = 0;
        };
      };
    };
  };
}
