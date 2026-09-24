{
  pkgs,
  lib,
  config,
  ...
}:

let
  cfg = config.userSettings.vpn;
in
{
  options = {
    userSettings.vpn = lib.mkOption {
      type = lib.types.enum [
        "proton"
        "none"
      ];
      default = "none";
      description = "The VPN";
    };
  };

  config = {
    home.packages = lib.optional (cfg == "proton") pkgs.proton-vpn;
  };
}
