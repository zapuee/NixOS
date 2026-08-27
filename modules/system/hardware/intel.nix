# Intel
{ config, lib, ... }:

lib.mkIf (config.systemSettings.hardware.gpu == "intel") {
}
