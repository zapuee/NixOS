{ inputs, ... }:

let
  lib = inputs.nixpkgs.lib;

  hostNames = builtins.attrNames (
    lib.filterAttrs (_name: type: type == "directory") (builtins.readDir ../hosts)
  );

  mkHost =
    name:
    let
      hostPath = ../hosts + "/${name}";
      metadata = import (hostPath + "/host.nix");
      host = {
        inherit name;
        system = metadata.system or "x86_64-linux";
        inherit (metadata) stateVersion username;
      };
    in
    lib.nameValuePair name (
      lib.nixosSystem {
        inherit (host) system;

        specialArgs = {
          inherit host inputs;
        };

        modules = [
          (inputs.import-tree ../modules/system)
          (hostPath + "/configuration.nix")
          (hostPath + "/hardware-configuration.nix")

          { networking.hostName = host.name; }

          inputs.nur.modules.nixos.default
          inputs.home-manager.nixosModules.home-manager

          {
            home-manager = {
              useGlobalPkgs = true;
              useUserPackages = true;
              extraSpecialArgs = {
                inherit host inputs;
              };
              users.${host.username}.imports = [
                (hostPath + "/home.nix")
                (inputs.import-tree ../modules/user)
              ];
            };
          }
        ];
      }
    );
in
{
  flake.nixosConfigurations = builtins.listToAttrs (map mkHost hostNames);
}
