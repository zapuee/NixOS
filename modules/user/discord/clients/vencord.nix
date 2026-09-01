{ lib, config, ... }:

let
  cssFiles = lib.filterAttrs
    (name: type:
      type == "regular" && lib.hasSuffix ".css" name
    )
    (builtins.readDir ../themes);

  themes = lib.mapAttrs
    (name: _: builtins.readFile ../themes/${name})
    cssFiles;

  enabledThemes = builtins.attrNames themes;
in
{
  config = lib.mkIf (
    config.userSettings.discord.client == "vencord" &&
    config.userSettings.discord.enable
  ) {
    programs.vesktop = {
      enable = true;

      settings = {
        discordBranch = "stable";
      };

      vencord = {
        useSystem = true;

        settings = {
          autoUpdate = false;
          notifyAboutUpdates = true;
          useQuickCss = true;

          enabledThemes = enabledThemes;

          plugins = {
            FakeNitro.enabled = true;
          };
        };

        themes = themes;
      };
    };
  };
}
