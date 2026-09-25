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
        spawn-at-startup = lib.optionals (config.userSettings.desktopShell == "noctalia") [ "noctalia" ];
      };

      # This host policy must follow Theme Manager's general window rules so
      # the more specific Picture-in-Picture geometry wins.
      extraConfig = lib.mkOrder 2000 ''
        window-rule {
          match app-id="firefox$" title="^Picture-in-Picture$"
          open-floating true
          geometry-corner-radius 0
          clip-to-geometry true
          draw-border-with-background false
        }
      '';
    };
  };
}
