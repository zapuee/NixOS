{ pkgs, config, lib, ... }:

{
  options = {
    systemSettings.bootloader = lib.mkOption {
      type = lib.types.enum [ "grub" "systemd" ];
      default = "grub";
      description = "bootloader/init system";
    };
  };

  config = {
    boot = {
      kernelPackages = pkgs.linuxPackages_latest;
      loader.efi.canTouchEfiVariables = true;
    };
  };
}
