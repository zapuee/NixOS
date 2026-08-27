{ lib, config, ... }:

let
  cfg = config.userSettings.equibop;

  # Find every .css file beside this module
  cssFiles = lib.filterAttrs
    (name: type:
      type == "regular" && lib.hasSuffix ".css" name
    )
    (builtins.readDir ./.);

  # Read every CSS file into an attribute set
  themes = lib.mapAttrs
    (name: _: builtins.readFile ./${name})
    cssFiles;

  # Automatically enable every discovered theme
  enabledThemes = builtins.attrNames themes;
in
{
  options.userSettings.equibop.enable = lib.mkOption {
    type = lib.types.bool;
    default = false;
    description = "Enable Equibop";
  };

  config = lib.mkIf cfg.enable {
    programs.equibop = {
      enable = true;

      settings = {
        discordBranch = "stable";
      };

      equicord = {
        settings = {
          enabledThemes = enabledThemes;
          autoUpdate = false;
          notifyAboutUpdates = true;
          useQuickCss = true;

          plugins = {
            FakeNitro.enabled = true;
          };
        };

        themes = themes;
      };
    };
  };
}
