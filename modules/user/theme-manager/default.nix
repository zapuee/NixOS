{
  config,
  lib,
  ...
}:

let
  cfg = config.userSettings.themeManager;
  project = ../../../pkgs/theme-manager;
  useNoctalia = config.userSettings.desktopShell == "noctalia";
in
{
  imports = [ ../../../pkgs/theme-manager/nix/home-manager-module.nix ];

  options.userSettings.themeManager.enable = lib.mkEnableOption "Theme Manager";

  config = lib.mkIf cfg.enable {
    programs.theme-manager = {
      enable = true;

      profiles = {
        glass = project + "/examples/profiles/glass.toml";
        paper = project + "/examples/profiles/paper.toml";
      };

      settings = {
        default_profile = "glass";

        provider =
          if useNoctalia then
            {
              kind = "luau:noctalia";
              inputs.palette = "$XDG_CACHE_HOME/theme-manager/noctalia-palette.json";
            }
          else
            {
              kind = "luau:static";
              settings.colors = {
                primary = "#80d998";
                outline = "#7c9598";
                error = "#ffb4ab";
                shadow = "#000000";
                secondary = "#9ccaff";
                surface = "#0f1512";
                on_surface = "#dfe4de";
              };
            };

        targets = {
          niri = {
            adapter = "luau:niri";
            outputs.config = "$XDG_CONFIG_HOME/theme-manager/generated/niri.kdl";
            settings = {
              global_role = "compositor";
              windows = [
                {
                  role = "browser";
                  app_id = "^firefox$";
                  use_opacity = true;
                }
                {
                  role = "terminal";
                  app_id = "^(foot|footclient)$";
                  use_opacity = false;
                }
              ];
            };
          };
          foot = {
            adapter = "luau:foot";
            outputs.config = "$XDG_CONFIG_HOME/theme-manager/generated/foot.ini";
            settings.role = "terminal";
          };
        };
      };
    };

    xdg.configFile = lib.mkIf useNoctalia {
      "theme-manager/integrations/noctalia/palette.template.json".source =
        project + "/integrations/noctalia/palette.template.json";
    };
  };
}
