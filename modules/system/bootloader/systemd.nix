{ config, lib, ... }:

{
  config = lib.mkIf (config.systemSettings.bootloader == "systemd") {
    boot = {
      loader = {
        grub.enable = false;
        systemd-boot = {
          enable = true;
          configurationLimit = 7;
        };
      };
    };
  };
}
