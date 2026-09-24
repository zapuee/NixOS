{ config, lib, ... }:

let
  browser = config.userSettings.browser;
  browserFiles = lib.filterAttrs (
    name: type: type == "regular" && name != "default.nix" && lib.hasSuffix ".nix" name
  ) (builtins.readDir ./.);
  browserNames = map (lib.removeSuffix ".nix") (builtins.attrNames browserFiles);
in
{
  options = {
    userSettings.browser = lib.mkOption {
      default = null;
      description = "Default browser";
      type = lib.types.nullOr (lib.types.enum browserNames);
    };
  };
  config = {
    programs.firefox.enable = browser == "firefox";
  };
}
