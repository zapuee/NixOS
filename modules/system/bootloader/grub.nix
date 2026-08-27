{ config, lib, ... }:

{
  config = lib.mkIf (config.systemSettings.bootloader == "grub") {
    boot = {
      loader = {
        systemd-boot.enable = false;
        grub = {
          enable = true;
          device = "nodev";
          efiSupport = true;
          useOSProber = true;
          configurationLimit = 7;
        };
      };
    };
  };
}
