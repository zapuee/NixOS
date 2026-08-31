{ lib, pkgs, config, ... }:

{
  config = lib.mkIf (config.userSettings.programming-cli.enable == true) {
    home.packages = with pkgs; [
      cargo
      rustc
      gcc
    ];
  };
}
