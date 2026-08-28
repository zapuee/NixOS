{ config, pkgs, lib, ... }:

{
  config = {
    # Packages
    environment.systemPackages = with pkgs; [ git gh ];

    # Locale and TZ
    services.timesyncd.enable = lib.mkDefault true;
    time.timeZone = "America/Chicago";
    i18n.defaultLocale = "en_US.UTF-8";
    i18n.extraLocaleSettings = {
      LC_ADDRESS = config.i18n.defaultLocale;
      LC_IDENTIFICATION = config.i18n.defaultLocale;
      LC_MEASUREMENT = config.i18n.defaultLocale;
      LC_MONETARY = config.i18n.defaultLocale;
      LC_NAME = config.i18n.defaultLocale;
      LC_NUMERIC = config.i18n.defaultLocale;
      LC_PAPER = config.i18n.defaultLocale;
      LC_TELEPHONE = config.i18n.defaultLocale;
      LC_TIME = config.i18n.defaultLocale;
    };

    nixpkgs.config.allowUnfree = true;
    
    # Ensure nix flakes are enabled
    nix.extraOptions = ''
      experimental-features = nix-command flakes
    '';

    home-manager.backupFileExtension = "backup";

    # Remove Bloat
    programs.nano.enable = lib.mkForce false;

    # Good for setting up systems
    programs.localsend.enable = true;
    programs.localsend.openFirewall = true;
  };
}
