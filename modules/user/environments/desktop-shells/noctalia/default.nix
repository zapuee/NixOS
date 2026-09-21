{ shared, config, inputs, lib, ... }:

let
  riceDir =
    "${config.home.homeDirectory}/NixOS/modules/user/environments/desktop-shells/noctalia/rices/${shared.hostname}";
in
{
  imports = [
    inputs.noctalia.homeModules.default
  ];

  config = lib.mkIf (config.userSettings.desktop-shell == "noctalia") {
    programs.noctalia.enable = true;

    home.file."Config-Swap".source =
      config.lib.file.mkOutOfStoreSymlink riceDir;
  };
}
