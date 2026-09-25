{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.theme-manager;
in
{
  imports = [
    ./integrations/foot.nix
    ./integrations/niri.nix
  ];

  options.programs.theme-manager = {
    enable = lib.mkEnableOption "theme-manager";

    package = lib.mkOption {
      type = lib.types.package;
      default = pkgs.callPackage ../package.nix { };
      defaultText = lib.literalExpression "pkgs.callPackage ./package.nix { }";
      description = "The theme-manager package to install and run.";
    };

    configFile = lib.mkOption {
      type = lib.types.path;
      description = "Typed Theme Manager TOML configuration.";
    };

    profiles = lib.mkOption {
      type = lib.types.attrsOf lib.types.path;
      default = { };
      description = "Structure Profile names mapped to TOML source files.";
    };

    enableService = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Watch runtime theme inputs during the graphical session.";
    };

    service = {
      logLevel = lib.mkOption {
        type = lib.types.str;
        default = "theme_manager=info";
        description = "RUST_LOG filter for the watcher service.";
      };
      restartSec = lib.mkOption {
        type = lib.types.str;
        default = "1s";
        description = "Delay before systemd restarts an unexpectedly failed watcher.";
      };
    };

    integrations = {
      niri = lib.mkOption {
        type = lib.types.bool;
        default = lib.attrByPath [ "wayland" "windowManager" "niri" "enable" ] false config;
        description = "Include Theme Manager's generated Niri fragment.";
      };
      foot = lib.mkOption {
        type = lib.types.bool;
        default = lib.attrByPath [ "programs" "foot" "enable" ] false config;
        description = "Include Theme Manager's generated Foot fragment.";
      };
    };
  };

  config = lib.mkIf cfg.enable {
    home.packages = [ cfg.package ];

    xdg.configFile = {
      "theme-manager/config.toml".source = cfg.configFile;
    }
    // lib.mapAttrs' (
      name: source: lib.nameValuePair "theme-manager/profiles/${name}.toml" { inherit source; }
    ) cfg.profiles;

    systemd.user.services.theme-manager = lib.mkIf cfg.enableService {
      Unit = {
        Description = "Live desktop theme watcher";
        After = [ "graphical-session.target" ];
        PartOf = [ "graphical-session.target" ];
      };

      Service = {
        ExecStart = "${cfg.package}/bin/theme-manager watch";
        Restart = "on-failure";
        RestartSec = cfg.service.restartSec;
        Environment = "RUST_LOG=${cfg.service.logLevel}";
      };

      Install.WantedBy = [ "graphical-session.target" ];
    };
  };
}
