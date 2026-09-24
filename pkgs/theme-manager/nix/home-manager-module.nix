{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.theme-manager;
  toml = pkgs.formats.toml { };

  providerType = lib.types.submodule {
    options = {
      kind = lib.mkOption {
        type = lib.types.str;
        description = "Namespaced provider adapter ID.";
      };
      inputs = lib.mkOption {
        type = lib.types.attrsOf lib.types.str;
        default = { };
        description = "Named provider input paths read by the Rust host.";
      };
      settings = lib.mkOption {
        inherit (toml) type;
        default = { };
        description = "Provider-specific settings passed to the adapter.";
      };
    };
  };

  targetType = lib.types.submodule {
    options = {
      adapter = lib.mkOption {
        type = lib.types.str;
        description = "Namespaced target adapter ID.";
      };
      enabled = lib.mkOption {
        type = lib.types.bool;
        default = true;
        description = "Whether this target may be used and toggled at runtime.";
      };
      outputs = lib.mkOption {
        type = lib.types.attrsOf lib.types.str;
        default = { };
        description = "Adapter output names mapped to generated file paths.";
      };
      settings = lib.mkOption {
        inherit (toml) type;
        default = { };
        description = "Target-specific settings passed to the adapter.";
      };
    };
  };

  settingsType = lib.types.submodule {
    freeformType = toml.type;
    options = {
      profiles_dir = lib.mkOption {
        type = lib.types.str;
        default = "$XDG_CONFIG_HOME/theme-manager/profiles";
        description = "Directory containing structure profiles.";
      };
      state_file = lib.mkOption {
        type = lib.types.str;
        default = "$XDG_STATE_HOME/theme-manager/state.toml";
        description = "Runtime state and ownership database.";
      };
      default_profile = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
        description = "Profile used before one has been selected explicitly.";
      };
      tui.vim_keys = lib.mkOption {
        type = lib.types.bool;
        default = true;
        description = "Enable Vim-style TUI navigation.";
      };
      extensions = {
        dirs = lib.mkOption {
          type = lib.types.listOf lib.types.str;
          default = [ ];
          description = "Additional extension discovery directories.";
        };
        limits = {
          memory_mib = lib.mkOption {
            type = lib.types.ints.between 4 256;
            default = 32;
          };
          execution_ms = lib.mkOption {
            type = lib.types.ints.between 10 5000;
            default = 250;
          };
          io_mib = lib.mkOption {
            type = lib.types.ints.between 1 64;
            default = 4;
          };
        };
      };
      provider = lib.mkOption {
        type = providerType;
        description = "Palette provider configuration.";
      };
      targets = lib.mkOption {
        type = lib.types.attrsOf targetType;
        default = { };
        description = "Configured target instances.";
      };
    };
  };

  removeNulls =
    value:
    if builtins.isAttrs value then
      lib.mapAttrs (_: removeNulls) (lib.filterAttrs (_: item: item != null) value)
    else if builtins.isList value then
      map removeNulls value
    else
      value;

  generatedConfig =
    if cfg.settings == null then
      null
    else
      toml.generate "theme-manager-config.toml" (removeNulls cfg.settings);
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

    configFile = lib.mkOption {
      type = lib.types.nullOr lib.types.path;
      default = null;
      description = "Raw config.toml escape hatch; mutually exclusive with settings.";
    };

    settings = lib.mkOption {
      type = lib.types.nullOr settingsType;
      default = null;
      description = "Typed configuration rendered to theme-manager/config.toml.";
    };

    profiles = lib.mkOption {
      type = lib.types.attrsOf lib.types.path;
      default = { };
      description = "Profile names mapped to TOML source files.";
    };

    enableService = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Start theme-manager's live watcher with the graphical session.";
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
        description = "Delay before systemd restarts a failed watcher.";
      };
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

  config = lib.mkIf cfg.enable (
    lib.mkMerge [
      {
        assertions = [
          {
            assertion = cfg.configFile == null || cfg.settings == null;
            message = "programs.theme-manager.configFile and settings are mutually exclusive";
          }
          {
            assertion = cfg.profiles == { } || cfg.settings != null || cfg.configFile != null;
            message = "programs.theme-manager.profiles requires settings or a matching configFile";
          }
        ];

        home.packages = [ cfg.package ];

        xdg.configFile = lib.mkMerge [
          (lib.mkIf (cfg.configFile != null) {
            "theme-manager/config.toml".source = cfg.configFile;
          })
          (lib.mkIf (generatedConfig != null) {
            "theme-manager/config.toml".source = generatedConfig;
          })
          (lib.mapAttrs' (
            name: source: lib.nameValuePair "theme-manager/profiles/${name}.toml" { inherit source; }
          ) cfg.profiles)
        ];
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
            RestartSec = cfg.service.restartSec;
            Environment = "RUST_LOG=${cfg.service.logLevel}";
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
    ]
  );
}
