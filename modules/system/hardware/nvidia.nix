{ config, lib, ... }:

lib.mkIf (config.systemSettings.hardware.gpu == "nvidia") {
  services.xserver.videoDrivers = [ "nvidia" ];

  hardware.graphics = {
    enable = true;
    enable32Bit = true;
  };

  hardware.nvidia = {
    open = false;
    modesetting.enable = true;
  };
}
