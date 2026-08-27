{ shared, config, inputs, pkgs, pkgs-stable, ... }:

{
  config = {
    userSettings = {

      editor = "nvf"; # nvf
      terminal = "alacritty"; # alacritty, kitty, foot
      browser = "firefox"; # firefox
      vpn = "proton"; # proton

      noctalia.enable = true; # for wayland: niri/hyprland ricing

      shell.enable = true; # enable misc zsh configs
      shell.apps.enable = true; # useful cli util
      shell.extraApps.enable = true; # fun cli like neofetch

      media.enable = true; # media-players like vlc
      spicetify.enable = true; # spotify but cooler
      
      starship.enable = true; # for cool terminal thoing
      starship.style = "tokyonight"; # list is in the starship directory

      equibop.enable = true; # for discord

    };

    home.username = shared.username;
    home.homeDirectory = "/home/" + shared.username;
  
    home.stateVersion = "26.05";
  };
}
