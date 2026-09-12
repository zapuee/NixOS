{ pkgs, lib, config, ... }:

let
  # Find every .css file beside this module
  clients = lib.filterAttrs
    (name: type:
      type == "regular" && lib.hasSuffix ".nix" name
    )
    (builtins.readDir ./clients);

  clientNames = map
    (name: lib.removeSuffix ".nix" name)
    (builtins.attrNames clients);
in
{
  options.userSettings.discord = {
    enable = lib.mkEnableOption "discord";
    client = lib.mkOption {
      type = lib.types.enum clientNames;
      default = "vencord";
      description = "Choose the client";
    };
  };

  config = {
    home.packages = with pkgs; [
      concord-tui
    ];
  };
}
