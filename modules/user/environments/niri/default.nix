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
