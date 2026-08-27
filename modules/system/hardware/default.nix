{ config, lib, ... }:

{
  options.systemSettings.hardware.gpu = lib.mkOption {
    type = lib.types.enum [ "nvidia" "none" ];
    default = "none";
  };

  config = {
    hardware.graphics.enable = true;
  };

  imports = [
    ./nvidia.nix
  ];
}
