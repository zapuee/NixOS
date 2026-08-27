{ osConfig, config, lib, ... }:

{
  config = lib.mkIf (osConfig.systemSettings.environment == builtins.baseNameOf ./.) {
    programs.wofi = {
      enable = true;

      settings = {
        width = 600;
        height = 400;
        show = "drun";
        prompt = "Applications";
      };
    };
  };
}
