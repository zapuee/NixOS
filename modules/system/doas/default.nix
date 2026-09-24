{
  host,
  config,
  lib,
  ...
}:

let
  cfg = config.systemSettings.doas;
in
{
  options.systemSettings.doas = {
    enable = lib.mkEnableOption "Replace sudo with doas";
  };

  config = lib.mkIf cfg.enable {
    security = {
      sudo.enable = false;
      doas = {
        enable = true;
        extraRules = [
          {
            users = [ host.username ];
            keepEnv = true;
            persist = true;
          }
        ];
      };
    };
  };
}
