{
  lib,
  pkgs,
  config,
  ...
}:

{
  config = lib.mkIf config.userSettings.development.enable {
    home.packages = with pkgs; [
      cargo
      rustc
      gcc
    ];
  };
}
