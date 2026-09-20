{ pkgs, pkgs-stable, shared, lib, config, ... }:


{
  config = {
    systemSettings = {
      bootloader = "systemd"; # grub, systemd
      environment = "niri"; # niri, plasma, hyprland

      flatpak.enable = true; # for flatpak packages
      hardware.gpu = "nvidia"; # nvidia, intel
      
      demucs.enable = true; # removing vocals off audio
      mail.enable = true; # tutomail desktop
      doas.enable = true; # sudo -> doas = less bloat
      keyd.enable = true; # for key remapping
      virtualization.enable = true; # for testing builds in a vm
      bluetooth.enable = true; # self explanitory
      games.enable = true; # bunch of launchers and sober
      ai.enable = true; # for ai stuff
    };

    # Assigning server user i think
    programs.zsh.enable = true;
    users.users.${shared.username} = {
      isNormalUser = true;
      description = shared.username;
      shell = pkgs.zsh;
      extraGroups = [
        "wheel"
        "networkmanager"
        "audio"
        "video"
        "render"
        "input"
        "storage"
      ];
    };

    networking.networkmanager.enable = true;
    system.stateVersion = "26.05";

    # for my acer monitor config 
    boot.kernelModules = [ "i2c-dev" ];
    environment.systemPackages = with pkgs; [ ddcutil ddccontrol ];
  };

}
