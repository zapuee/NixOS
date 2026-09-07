{ pkgs, lib, config, ... }:

{
  options.userSettings.bitwarden = {
    enable = lib.mkEnableOption "bitwarden";
  };

  config = lib.mkIf config.userSettings.bitwarden.enable {
    programs.rbw = {
      enable = true;
      settings = {
        email = "zapuee@tutamail.com";
        pinentry = pkgs.pinentry-curses;
      };
    };
  };
}
