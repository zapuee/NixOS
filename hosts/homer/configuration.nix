{ pkgs, pkgs-stable, shared, lib, config, ... }:


{
  config = {
    systemSettings = {
      bootloader = "grub"; # grub, systemd
      environment = "niri"; # niri, plasma, hyprland

      flatpak.enable = true; # for flatpak packages
      hardware.gpu = "nvidia"; # nvidia
      
      doas.enable = true; # sudo -> doas = less bloat
      keyd.enable = true; # for key remapping
      virtualization.enable = true; # for testing builds in a vm
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

    system.stateVersion = "26.05";
  };

}
