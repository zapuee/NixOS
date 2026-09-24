{ lib, config, ... }:

let
  # Find every .css file in ../themes/
  cssFiles = lib.filterAttrs (name: type: type == "regular" && lib.hasSuffix ".css" name) (
    builtins.readDir ../themes
  );

  # Read every CSS file into an attribute set
  themes = lib.mapAttrs (name: _: builtins.readFile ../themes/${name}) cssFiles;

  # Automatically enable every discovered theme
  enabledThemes = builtins.attrNames themes;
in
{
  config =
    lib.mkIf (config.userSettings.discord.client == "equibop" && config.userSettings.discord.enable)
      {
        programs.equibop = {
          enable = true;

          settings = {
            discordBranch = "stable";
          };

          equicord = {
            settings = {
              inherit enabledThemes;
              autoUpdate = false;
              notifyAboutUpdates = true;
              useQuickCss = true;

              plugins = {
                FakeNitro.enabled = true;
              };
            };

            inherit themes;
          };
        };
      };
}
