{
  config,
  pkgs,
  lib,
  ...
}:

{
  config = {
    environment.systemPackages = with pkgs; [
      git
      gh
      tor-browser
    ];
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

    fonts.packages = with pkgs; [
      noto-fonts
      noto-fonts-cjk-sans
      noto-fonts-color-emoji
    ];

    nix.settings.experimental-features = [
      "nix-command"
      "flakes"
    ];

    home-manager.backupFileExtension = "backup";

    programs = {
      # Nano is disabled because the configured editor is supplied by Home Manager.
      nano.enable = lib.mkForce false;

      localsend = {
        enable = true;
        openFirewall = true;
      };
    };
  };
}
