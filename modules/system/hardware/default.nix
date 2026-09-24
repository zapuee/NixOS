{ lib, ... }:

{
  options.systemSettings.hardware.gpu = lib.mkOption {
    type = lib.types.enum [
      "intel"
      "nvidia"
      "none"
    ];
    default = "none";
  };

  config = {
    hardware.graphics.enable = true;
  };
}
