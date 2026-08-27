{ pkgs, osConfig, lib, ... }:

let
  cfg = osConfig.systemSettings.environment;

  environments =
    lib.filterAttrs
      (_name: type: type == "directory")
      (builtins.readDir ./.);

  environmentNames = builtins.attrNames environments;
in
{
  options.systemSettings.environment = lib.mkOption {
    type = lib.types.enum environmentNames;
    default = "plasma";
    description = "Choice of DE/WM";
  };
}
