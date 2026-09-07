{ pkgs, lib, config, ... }:

{
  options.systemSettings.mail = {
    enable = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "for tutamail etc";
    };
  };

  config = lib.mkIf (config.systemSettings.mail.enable == true) {
    environment.systemPackages = with pkgs; [
      tutanota-desktop
    ];
  };

}
