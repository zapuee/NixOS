{ lib, ... }:

let
  shellDirectories = lib.filterAttrs (_name: type: type == "directory") (builtins.readDir ./.);

  shellNames = lib.attrNames shellDirectories;
in
{
  options.userSettings.desktopShell = lib.mkOption {
    type = lib.types.nullOr (lib.types.enum shellNames);
    default = null;
    description = "Optional desktop shell to start with the compositor.";
  };
}
