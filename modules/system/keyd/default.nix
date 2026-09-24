{ lib, config, ... }:

let
  cfg = config.systemSettings.keyd;
in
{
  options.systemSettings.keyd.enable = lib.mkEnableOption "system-wide key remapping with keyd";

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
