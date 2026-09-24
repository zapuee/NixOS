{
  pkgs,
  lib,
  config,
  ...
}:

{
  options.systemSettings.email.enable = lib.mkEnableOption "desktop email clients";

  config = lib.mkIf config.systemSettings.email.enable {
    environment.systemPackages = with pkgs; [
      tutanota-desktop
    ];
  };

}
