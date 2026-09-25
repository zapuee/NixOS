{ config, lib, ... }:
let
  cfg = config.programs.theme-manager;
in
{
  config = lib.mkIf (cfg.enable && cfg.integrations.niri) {
    wayland.windowManager.niri.extraConfig = lib.mkAfter ''
      include optional=true "~/.config/theme-manager/generated/niri.kdl"
    '';
  };
}
