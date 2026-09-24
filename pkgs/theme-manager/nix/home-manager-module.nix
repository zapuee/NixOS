{ config, lib, pkgs, ... }:

let
  cfg = config.programs.theme-manager;
in
{
  options.programs.theme-manager = {
    enable = lib.mkEnableOption "theme-manager";

    package = lib.mkOption {
      type = lib.types.package;
      default = pkgs.callPackage ../package.nix { };
      defaultText = lib.literalExpression "pkgs.callPackage ./package.nix { }";
      description = "The theme-manager package to install and run.";
    };

    enableService = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Start theme-manager's live watcher with the graphical session.";
    };

    niriIntegration = lib.mkOption {
      type = lib.types.bool;
      default = lib.attrByPath [ "wayland" "windowManager" "niri" "enable" ] false config;
      description = "Include theme-manager's generated fragment in Niri.";
    };

    footIntegration = lib.mkOption {
      type = lib.types.bool;
      default = lib.attrByPath [ "programs" "foot" "enable" ] false config;
      description = "Include theme-manager's generated fragment in Foot.";
    };
  };

  config = lib.mkIf cfg.enable (lib.mkMerge [
    {
      home.packages = [ cfg.package ];
    }

    (lib.mkIf cfg.enableService {
      systemd.user.services.theme-manager = {
        Unit = {
          Description = "Live desktop theme watcher";
          After = [ "graphical-session.target" ];
          PartOf = [ "graphical-session.target" ];
        };

        Service = {
          ExecStart = "${cfg.package}/bin/theme-manager watch";
          Restart = "on-failure";
          RestartSec = 1;
        };

        Install.WantedBy = [ "graphical-session.target" ];
      };
    })

    (lib.mkIf cfg.niriIntegration {
      wayland.windowManager.niri.extraConfig = lib.mkAfter ''
        include optional=true "~/.config/theme-manager/generated/niri.kdl"
      '';
    })

    (lib.mkIf cfg.footIntegration {
      programs.foot.settings.main.include = lib.mkAfter [
        "~/.config/theme-manager/generated/foot.ini"
      ];
    })
  ]);
}
