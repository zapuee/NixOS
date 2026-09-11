{ lib, pkgs, config, ... }:

let
  shellDirectories =
    lib.filterAttrs
      (_name: type: type == "directory")
      (builtins.readDir ./.);

  shellNames = lib.attrNames shellDirectories;
in
{
  options.userSettings.desktop-shell = lib.mkOption {
    type = lib.types.enum shellNames;
    default = "noctalia";
    description = "Desktop shell to use.";
  };
}
