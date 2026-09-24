{ shared, config, inputs, pkgs, pkgs-stable, ... }:

{
  config = {
    userSettings = {

      editor = "nvf"; # nvf
      terminal = "foot"; # alacritty, kitty, foot
      browser = "firefox"; # firefox
      vpn = "proton"; # proton
      bitwarden.enable = true; # password manager (CLI)
      theme-manager.enable = true;

      desktop-shell = "noctalia"; # noctalia, serpantinum

      shell.enable = true; # enable misc zsh configs
      shell.apps.enable = true; # useful cli util
      shell.extraApps.enable = true; # fun cli like neofetch

      tmux.enable = true; # tmux obv

      programming-cli.enable = true; # stuff like cargo and whatnot

      obsidian.enable = true; # for note taking
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


    # host some wallpapers (for stylus)
    systemd.user.services.wallpaper-server = {
      Unit = {
        Description = "Local wallpaper server";
      };

      Service = {
        ExecStart = "${pkgs.python3}/bin/python -m http.server 8080 --bind 127.0.0.1";
        WorkingDirectory = "%h/Pictures";
        Restart = "on-failure";
      };

      Install = {
        WantedBy = [ "default.target" ];
      };
    };

  };
}
