{ lib, config, ... }:

{
  options.userSettings.programming-cli.enable = lib.mkOption {
    type = lib.types.bool;
    default = false;
    description = "for programming cli";
  };
}
