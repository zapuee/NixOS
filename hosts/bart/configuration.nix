{ pkgs, pkgs-stable, shared, lib, config, ... }:


{
  config = {
    systemSettings = {
      bootloader = "systemd"; # grub, systemd
      environment = "niri"; # niri, plasma, hyprland

      flatpak.enable = true; # for flatpak packages
      hardware.gpu = "intel"; # nvidia intel
      
      mail.enable = true; # tutomail desktop
      doas.enable = true; # sudo -> doas = less bloat
      keyd.enable = true; # for key remapping
      virtualization.enable = false; # for testing builds in a vm
      bluetooth.enable = true; # self explanitory
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

    boot.loader.systemd-boot.configurationLimit = lib.mkForce 5;
    networking.networkmanager.enable = true;
    system.stateVersion = "26.05";
  };

}
