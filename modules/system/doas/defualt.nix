{ shared, config, lib, ... }:

let
  cfg = config.systemSettings.doas;
in
{
  options.systemSettings.doas = {
    enable = lib.mkEnableOption "Replace sudo with doas";
  };

  config = lib.mkIf cfg.enable {
    security.doas.enable = true;
    security.sudo.enable = false;

    security.doas.extraRules = [
      {
        users = [ shared.username ];
        keepEnv = true;
        persist = true;
      }
    ];
  };
}
