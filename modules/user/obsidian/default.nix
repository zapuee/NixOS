{ lib, config, ... }:

{
  options.userSettings.obsidian.enable =
    lib.mkEnableOption "Enable note taking app obsidian";

  config = lib.mkIf config.userSettings.obsidian.enable {
    programs.obsidian = {
      enable = true;
      cli.enable = true;
    };
  };
}
