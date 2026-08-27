{ osConfig, config, inputs, lib, ... }:

# NOTE: Noctalia requires systemd

{
  imports = [
    inputs.noctalia.homeModules.default
  ];

  options = {
    userSettings.noctalia.enable = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "replacement of many hyprland thangs";
    };
  };

  config = lib.mkIf (config.userSettings.noctalia.enable) {
    home.file.".config/noctalia/config.toml".source = ./noctalia-config.toml;

    programs.noctalia = {
      enable = true;
    };
  };
}
