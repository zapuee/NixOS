{ inputs, pkgs, osConfig, lib, ... }:

let
  cfg = osConfig.systemSettings.environment;
in
{
  config = lib.mkIf (cfg == builtins.baseNameOf ./.) {
    home.packages = with pkgs; [
      grim
      slurp
    ];

    wayland.windowManager.niri = {
      enable = true;

      settings = {
        input.keyboard = {
          repeat-delay = 300;
          repeat-rate = 60;
        };
        spawn-at-startup = [
          "noctalia"
        ];
        layout = {
          focus-ring = {
            width = 3;
            active-color = "#c9b45880";
            inactive-color = "#5c563060";
            urgent-color = "#8f7d3280";
          };
        };

        window-rule._children = [
          {
            geometry-corner-radius = 0;
            clip-to-geometry = true;
          }
        ];
      };
    };
  };
}
