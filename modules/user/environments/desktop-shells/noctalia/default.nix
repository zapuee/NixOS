{ shared, osConfig, config, inputs, lib, ... }:

# NOTE: Noctalia requires systemd

{
  imports = [
    inputs.noctalia.homeModules.default
  ];

  config = lib.mkIf (config.userSettings.desktop-shell == "noctalia") {
    home.file.".config/noctalia/config.toml".source =
      ./${shared.hostname}.toml;

    programs.noctalia = {
      enable = true;
    };
  };
}
