{ config, lib, ... }:

let
  cfg = config.systemSettings.bluetooth.enable;
in
{
  options.systemSettings.bluetooth.enable = lib.mkOption {
    type = lib.types.bool;
    default = false;
    description = "bluueoth";
  };

  config = lib.mkIf cfg {
    hardware.bluetooth.enable = true;
    services.blueman.enable = true;
  };
}
