{ pkgs, ... }:
{
  config = {
    systemSettings = {
      bootloader = "systemd"; # grub, systemd
      desktop = "niri";

      flatpak.enable = true; # for flatpak packages
      hardware.gpu = "nvidia"; # nvidia, intel

      demucs.enable = true; # removing vocals off audio
      email.enable = true;
      doas.enable = true; # sudo -> doas = less bloat
      keyd.enable = true; # for key remapping
      virtualization.enable = true; # for testing builds in a vm
      bluetooth.enable = true; # self explanitory
      games.enable = true; # bunch of launchers and sober
      aiTools.enable = true;
      printing.enable = true;
    };

    networking.networkmanager.enable = true;

    # Support DDC/CI controls for the external monitor.
    boot.kernelModules = [ "i2c-dev" ];
    environment.systemPackages = with pkgs; [
      ddcutil
      ddccontrol
    ];
  };

}
