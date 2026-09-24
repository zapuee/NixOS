{ lib, ... }:

let
  editorFiles = lib.filterAttrs (
    name: type: type == "regular" && name != "default.nix" && lib.hasSuffix ".nix" name
  ) (builtins.readDir ./.);
  editorNames = map (lib.removeSuffix ".nix") (builtins.attrNames editorFiles);
in
{
  options.userSettings.editor = lib.mkOption {
    default = "nvf";
    description = "Code editor to configure.";
    type = lib.types.enum editorNames;
  };
}
