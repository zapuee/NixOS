{ config, lib, ... }:

lib.mkIf (config.systemSettings.hardware.gpu == "nvidia") {
  services.xserver.videoDrivers = [ "nvidia" ];

  hardware.nvidia = {
    open = false;
    modesetting.enable = true;
  };
}
