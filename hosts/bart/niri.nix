{ lib, osConfig, ... }:

{
  config = lib.mkIf (osConfig.systemSettings.desktop == "niri") {
    wayland.windowManager.niri.settings = {
      layout.border.off = { };
    };
  };
}
