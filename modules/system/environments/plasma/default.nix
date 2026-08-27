{ config, lib, ... }:

{
  config = lib.mkIf (config.systemSettings.environment == builtins.baseNameOf ./.) {
    services.xserver.enable = true;
    services.displayManager.sddm.enable = true;
    services.displayManager.sddm.wayland.enable = true;
    services.desktopManager.plasma6.enable = true;
  };
}
