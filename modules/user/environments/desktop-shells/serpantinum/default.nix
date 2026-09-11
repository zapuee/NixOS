{ shared, osConfig, config, inputs, lib, ... }:

{
  config = lib.mkIf (config.userSettings.desktop-shell == "serpantinum") {

  };
}
