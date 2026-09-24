{
  pkgs,
  lib,
  config,
  ...
}:

{
  options.userSettings.development.enable = lib.mkEnableOption "command-line development tools";

  config = lib.mkIf config.userSettings.development.enable {
    home.packages = with pkgs; [
      python3
    ];
  };
}
