{
  pkgs,
  lib,
  config,
  inputs,
  ...
}:

{
  imports = [
    inputs.spicetify-nix.homeManagerModules.spicetify
  ];

  options.userSettings.spotify.enable = lib.mkEnableOption "Spotify with Spicetify customizations";

  config = lib.mkIf config.userSettings.spotify.enable {
    programs.spicetify = {
      enable = true;

      enabledExtensions =
        with inputs.spicetify-nix.legacyPackages.${pkgs.stdenv.hostPlatform.system}.extensions; [
          adblockify
          hidePodcasts
          shuffle
        ];

      theme = inputs.spicetify-nix.legacyPackages.${pkgs.stdenv.hostPlatform.system}.themes.catppuccin;
      colorScheme = "mocha";
    };
  };
}
