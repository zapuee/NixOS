{ ... }:
{
  imports = [ ./niri.nix ];

  config = {
    userSettings = {

      editor = "nvf";
      terminal = "foot";
      browser = "firefox";
      vpn = "proton";
      bitwarden = {
        enable = true;
        email = "zapuee@tutamail.com";
      };
      themeManager.enable = true;

      desktopShell = "noctalia";

      shell = {
        enable = true;
        utilities.enable = true;
        fun.enable = true;
      };

      tmux.enable = true;

      development.enable = true;

      obsidian.enable = true;
      media.enable = true;
      spotify.enable = true;

      starship = {
        enable = true;
        style = "backrooms";
        useNoctalia = true;
      };

      discord = {
        enable = true;
        client = "vencord";
      };

      wallpaperServer = {
        enable = true;
        directory = "Pictures";
      };
    };
  };
}
