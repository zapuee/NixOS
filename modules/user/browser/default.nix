{ config, lib, ... }:

let
  browser = config.userSettings.browser;
in {
  options = {
    userSettings.browser = lib.mkOption {
      default = null;
      description = "Default browser";
      type = lib.types.enum [ "firefox" null ];
    };
 };
  config = {
    programs.firefox.enable = lib.mkIf (browser == "firefox") true;
  };
}
