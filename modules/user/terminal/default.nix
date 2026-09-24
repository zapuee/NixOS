{ lib, ... }:

let
  terminalFiles = lib.filterAttrs (
    name: type: type == "regular" && name != "default.nix" && lib.hasSuffix ".nix" name
  ) (builtins.readDir ./.);
  terminalNames = map (lib.removeSuffix ".nix") (builtins.attrNames terminalFiles);
in
{
  options = {
    userSettings.terminal = lib.mkOption {
      default = "kitty";
      description = "Default terminal";
      type = lib.types.enum terminalNames;
    };
  };
}
