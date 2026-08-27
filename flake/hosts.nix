{ inputs, ... }:

let
  lib = inputs.nixpkgs.lib;

  hosts = builtins.filter (x: x != null) (
    lib.mapAttrsToList
      (name: value:
        if value == "directory"
        then name
        else null
      )
      (builtins.readDir ../hosts)
  );
in
{
  flake.nixosConfigurations = builtins.listToAttrs (
    map
      (host:
        let
          hostPath = ../hosts + "/${host}";
          shared = import (hostPath + "/shared.nix");
        in
        {
          name = host;

          value = lib.nixosSystem {
            system = "x86_64-linux";

            specialArgs = {
              inherit inputs shared;
            };

            modules = [
              (inputs.import-tree ../modules/system)

              hostPath

              {
                networking.hostName = host;
              }

              inputs.nur.modules.nixos.default

              {
                nixpkgs.overlays = [
                  (final: prev: {
                    stable = import inputs.nixpkgs-stable {
                      inherit (final.stdenv.hostPlatform) system;
                      inherit (final) config;
                    };
                  })
                ];
              }

              inputs.home-manager.nixosModules.home-manager

              {
                home-manager.useGlobalPkgs = true;
                home-manager.useUserPackages = true;

                home-manager.extraSpecialArgs = {
                  inherit inputs shared;
                };
              }
            ];
          };
        }
      )
      hosts
  );
}
