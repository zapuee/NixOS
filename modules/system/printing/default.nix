{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.systemSettings.printing;
in
{
  options.systemSettings.printing.enable = lib.mkEnableOption "printing and network printer discovery";

  config = lib.mkIf cfg.enable {
    services.printing = {
      enable = true;
      drivers = [ pkgs.brlaser ];
    };

    services.avahi = {
      enable = true;
      nssmdns4 = true;
    };
  };
}
