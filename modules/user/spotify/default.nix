{ pkgs, lib, config, inputs, ... }:

{
  imports = [
    inputs.spicetify-nix.homeManagerModules.spicetify
  ];

  options.userSettings.spicetify.enable =
    lib.mkEnableOption "Enable Spicetify";

  config = lib.mkIf config.userSettings.spicetify.enable {
    programs.spicetify = {
      enable = true;

      enabledExtensions = with inputs.spicetify-nix.legacyPackages.${pkgs.system}.extensions; [
        adblockify
        hidePodcasts
        shuffle
      ];

      theme = inputs.spicetify-nix.legacyPackages.${pkgs.system}.themes.catppuccin;
      colorScheme = "mocha";
    };
  };
}
