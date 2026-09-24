{ config, lib, ... }:

let
  cfg = config.systemSettings.bluetooth.enable;
in
{
  options.systemSettings.bluetooth.enable = lib.mkEnableOption "Bluetooth support and the Blueman applet";

  config = lib.mkIf cfg {
    hardware.bluetooth.enable = true;
    services.blueman.enable = true;
  };
}
