{
  config,
  pkgs,
  osConfig,
  lib,
  ...
}:

let
  cfg = osConfig.systemSettings.desktop;
in
{
  config = lib.mkIf (cfg == builtins.baseNameOf ./.) {
    home.packages = with pkgs; [
      grim
      slurp
      wayland
      libxkbcommon
    ];

    wayland.windowManager.niri = {
      enable = true;
      settings = {
        prefer-no-csd = { };
        input.keyboard = {
          repeat-delay = 300;
          repeat-rate = 60;
        };
        spawn-at-startup = lib.optional (config.userSettings.desktopShell == "noctalia") [ "noctalia" ];
        _children = [
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
