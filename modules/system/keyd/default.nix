{ lib, config, ... }:

let
  cfg = config.systemSettings.keyd;
in
{
  options.systemSettings.keyd = {
    enable = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "For key remapping";
    };
  };

  config = lib.mkIf cfg.enable {
    services.keyd = {
      enable = true;

      keyboards = {
        default = {
          ids = [ "*" ];

          settings = {
            main = {
              capslock = "esc";
            };
          };
        };
      };
    };
  };
}
