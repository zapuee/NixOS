{
  config,
  lib,
  ...
}:

let
  cfg = config.userSettings.themeManager;
  catalogDirectory = ../../../pkgs/theme-manager/profiles;
  profiles =
    lib.mapAttrs'
      (file: _type: lib.nameValuePair (lib.removeSuffix ".toml" file) (catalogDirectory + "/${file}"))
      (
        lib.filterAttrs (file: type: type == "regular" && lib.hasSuffix ".toml" file) (
          builtins.readDir catalogDirectory
        )
      );
in
{
  imports = [ ../../../pkgs/theme-manager/nix/home-manager-module.nix ];

  options.userSettings.themeManager.enable = lib.mkEnableOption "Theme Manager";

  config = lib.mkIf cfg.enable {
    programs.theme-manager = {
      enable = true;
      inherit profiles;
      configFile = ./config.toml;
    };
  };
}
