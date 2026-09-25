{ config, lib, ... }:
let
  cfg = config.programs.theme-manager;
in
{
  config = lib.mkIf (cfg.enable && cfg.integrations.foot) {
    programs.foot.settings.main.include = lib.mkAfter [
      "~/.config/theme-manager/generated/foot.ini"
    ];
  };
}
