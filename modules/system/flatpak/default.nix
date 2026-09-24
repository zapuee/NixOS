{
  inputs,
  pkgs,
  lib,
  config,
  ...
}:

let
  cfg = config.systemSettings.flatpak;
in
{

  imports = [ inputs.nix-flatpak.nixosModules.nix-flatpak ];

  options = {
    systemSettings.flatpak = {
      enable = lib.mkEnableOption "Enable flatpaks";
    };
  };

  config = lib.mkIf cfg.enable {
    services.flatpak = {
      enable = true;
      packages = [
        (
          let
            sha256 = "sha256-mFCGbrMCiaL2TQ+BOZId3G1+vIlSKHvBq7oGsfMIBYA=";
          in
          {
            appId = "io.github.zxcloli666.SoundcloudDesktop";
            inherit sha256;

            bundle = "${pkgs.fetchurl {
              url = "https://github.com/zxcloli666/SoundCloud-Desktop-EN/releases/download/8.4.9/soundcloud-desktop.flatpak";
              inherit sha256;
            }}";
          }
        )
      ]
      ++ lib.optionals config.systemSettings.games.enable [
        # games
        "org.vinegarhq.Sober"
        "com.usebottles.bottles"
      ];

      uninstallUnmanaged = true;
      update.auto = {
        enable = true;
        onCalendar = "weekly";
      };
    };

    xdg.portal = {
      enable = true;
      extraPortals = [
        pkgs.xdg-desktop-portal-gtk
      ];
      config.common.default = "gtk";
    };
  };
}
