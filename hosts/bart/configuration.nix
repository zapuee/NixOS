{ lib, ... }:
{
  config = {
    systemSettings = {
      bootloader = "systemd"; # grub, systemd
      desktop = "niri";

      flatpak.enable = true; # for flatpak packages
      hardware.gpu = "intel"; # nvidia intel

      demucs.enable = true; # removing vocals off audio
      email.enable = true;
      doas.enable = true; # sudo -> doas = less bloat
      keyd.enable = true; # for key remapping
      virtualization.enable = false; # for testing builds in a vm
      bluetooth.enable = true; # self explanitory
      games.enable = true; # bunch of launchers and sober
      aiTools.enable = true;
      printing.enable = true;
    };

    boot.loader.systemd-boot.configurationLimit = lib.mkForce 5;
    networking.networkmanager.enable = true;
  };
}
