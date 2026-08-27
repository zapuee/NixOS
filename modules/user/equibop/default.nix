{ lib, config, ... }:

let
  cfg = config.userSettings.equibop;
in
{
  options = {
    userSettings.equibop = {
      enable = lib.mkOption {
        type = lib.types.bool;
        default = false;
        description = "discord option";
      };
    };
  };

  config = lib.mkIf cfg.enable {
    programs.equibop = {
      enable = true;
      settings = {
        discordBranch = "stable";
      };

      equicord = {
        settings = {
          enabledThemes = [ "Dark-Matter.css" ];
          autoUpdate = false;
          notifyAboutUpdates = true;
          useQuickCss = true;
          plugins = {
            FakeNitro.enabled = true;
          };
        };
        themes = {
          "Dark-Matter.css" = ''
            @import url('https://DiscordStyles.github.io/DarkMatter/src/base.css');

            /* Variables */
            :root {
              --avatar-size: 32px;
              --background-image: url('https://i.imgur.com/7SbtKvw.png');
              --home-image: url('https://i.imgur.com/233d55Y.gif');
              --background-solid: #161921;
              --background-solid-dark: #101218;
              --background-solid-darker: #0c0e12;
              --accent: 37, 172, 232;
              --accent-alt: 29, 101, 134;
            }
          '';
        };
      };
    };
  };
}


