{ lib, config, ... }:

let
  cfg = config.userSettings.starship;

  # Find every .toml file in this directory
  tomlFiles = lib.filterAttrs
    (name: type:
      type == "regular" && lib.hasSuffix ".toml" name
    )
    (builtins.readDir ./.);

  # Turn "tokyonight.toml" -> "tokyonight"
  themes = map
    (name: lib.removeSuffix ".toml" name)
    (builtins.attrNames tomlFiles);
in
{
  options.userSettings.starship = {
    enable = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Enable Starship prompt";
    };

    style = lib.mkOption {
      type = lib.types.enum themes;
      default = "tokyonight";
      description = "The Starship theme";
    };
  };

  config = lib.mkIf cfg.enable {
    programs.starship = {
      enable = true;
      enableZshIntegration = true;
      settings = builtins.fromTOML(builtins.readFile ./${cfg.style}.toml);
    };
  };
}
