{ lib, ... }:

let
  desktops = lib.filterAttrs (_name: type: type == "directory") (builtins.readDir ./.);

  desktopNames = builtins.attrNames desktops;
in
{
  options.systemSettings.desktop = lib.mkOption {
    type = lib.types.enum desktopNames;
    default = "hyprland";
    description = "Desktop environment or window manager to enable.";
  };
}
