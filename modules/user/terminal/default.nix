{ config, lib, ... }:

{
  options = {
    userSettings.terminal = lib.mkOption {
      default = "kitty";
      description = "Default terminal";
      type = lib.types.enum [ "alacritty" "kitty" "foot" ];
    };
  };

  config = {
    userSettings.alacritty.enable = lib.mkDefault (config.userSettings.terminal == "alacritty");
    userSettings.kitty.enable = lib.mkDefault (config.userSettings.terminal == "kitty");
    userSettings.foot.enable = lib.mkDefault (config.userSettings.terminal == "foot");
  };
}
