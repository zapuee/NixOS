{ pkgs, osConfig, lib, ... }:

let
  cfg = osConfig.systemSettings.environment;
in
{
  config = lib.mkIf (cfg == builtins.baseNameOf ./.) {
    wayland.windowManager.niri = {
      extraConfig = ''
        spawn-at-startup "noctalia"
      '';
    };
  };
}
