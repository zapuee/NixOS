{ shared, config, inputs, pkgs, pkgs-stable, ... }:

{
  config = {
    userSettings = {

      editor = "nvf"; # nvf
      terminal = "foot"; # alacritty, kitty, foot
      browser = "firefox"; # firefox
      vpn = "proton"; # proton
      bitwarden.enable = true; # password manager (CLI)

      noctalia.enable = true; # for wayland: niri/hyprland ricing

      shell.enable = true; # enable misc zsh configs
      shell.apps.enable = true; # useful cli util
      shell.extraApps.enable = true; # fun cli like neofetch

      tmux.enable = true; # tmux obv

      programming-cli.enable = true; # stuff like cargo and whatnot

      media.enable = true; # media-players like vlc
      spicetify.enable = true; # spotify but cooler
      
      starship.enable = true; # for cool terminal thoing
      starship.style = "backrooms"; # list is in the starship directory
      starship.useNoctalia = true; # noctalia wallpaper defines style

      discord.enable = true;
      discord.client = "vencord"; # vencord, equibop

    };

    home.username = shared.username;
    home.homeDirectory = "/home/" + shared.username;
  
    home.stateVersion = "26.05";
  };
}
