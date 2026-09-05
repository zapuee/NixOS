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
        prefer-no-csd = {};
        input.keyboard = {
          repeat-delay = 300;
          repeat-rate = 60;
        };
        spawn-at-startup = [
          "noctalia"
        ];
        _children = [
          {
            window-rule = {
              match._props = {
                app-id = "firefox$";
              };
        
              opacity = 0.835;
              background-effect = {
                blur = true;
              };
            };
          }
          {
            window-rule = {
              match._props = {
                app-id = "firefox$";
                title = "^Picture-in-Picture$";
              };
        
              open-floating = true;
              geometry-corner-radius = 0;
              clip-to-geometry = true;
              draw-border-with-background = false;
            };
          }
        ];
      };
    };
  };
}
