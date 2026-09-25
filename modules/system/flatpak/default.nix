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
        "org.vinegarhq.Sober"
        "com.usebottles.bottles"
      ];

      uninstallUnmanaged = true;

      update.auto = {
        enable = true;
        onCalendar = "weekly";
      };
    };

    home-manager.users.zap =
      { lib, pkgs, ... }:
      {
        home.activation.soberConfig = lib.hm.dag.entryAfter [ "writeBoundary" ] ''
          mkdir -p "$HOME/.var/app/org.vinegarhq.Sober/config/sober"

          rm -f "$HOME/.var/app/org.vinegarhq.Sober/config/sober/config.json"

          install -m 600 \
              ${
                pkgs.writeText "sober-config.json" (
                  builtins.toJSON {
                    allow_gamepad_permission = false;
                    close_on_leave = false;

                    discord_rpc_enabled = true;
                    discord_rpc_show_join_button = false;

                    enable_gamemode = true;
                    enable_hidpi = false;
                    enable_mobile_home_screen = false;

                    graphics_optimization_mode = "performance";

                    use_opengl = false;
                    touch_mode = "off";
                    use_console_experience = false;
                    use_libsecret = false;

                    fflags = {
                      DFFlagTextureQualityOverrideEnabled = true;
                      DFIntTextureQualityOverride = 1;

                      DFFlagDebugPauseVoxelizer = true;

                      FIntFRMMinGrassDistance = 0;
                      FIntFRMMaxGrassDistance = 0;
                    };
                  }
                )
              } \
            "$HOME/.var/app/org.vinegarhq.Sober/config/sober/config.json"
        '';
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
