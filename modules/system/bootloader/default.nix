{
  pkgs,
  lib,
  ...
}:

{
  options = {
    systemSettings.bootloader = lib.mkOption {
      type = lib.types.enum [
        "grub"
        "systemd"
      ];
      default = "grub";
      description = "Boot loader used by the host.";
    };
  };

  config = {
    boot = {
      kernelPackages = pkgs.linuxPackages_latest;
      loader.efi.canTouchEfiVariables = true;
    };
  };
}
