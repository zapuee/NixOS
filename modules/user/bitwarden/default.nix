{
  pkgs,
  lib,
  config,
  ...
}:

{
  options.userSettings.bitwarden = {
    enable = lib.mkEnableOption "bitwarden";

    email = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = "Email address used by rbw.";
    };
  };

  config = lib.mkIf config.userSettings.bitwarden.enable {
    programs.rbw = {
      enable = true;
      settings = {
        pinentry = pkgs.pinentry-curses;
      }
      // lib.optionalAttrs (config.userSettings.bitwarden.email != null) {
        inherit (config.userSettings.bitwarden) email;
      };
    };
  };
}
