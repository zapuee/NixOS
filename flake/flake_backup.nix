{
  description = "Tryna learn flakes";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    nixpkgs-stable.url = "nixpkgs/nixos-25.11";
    import-tree.url = "github:denful/import-tree";

    stylix = {
      url = "github:nix-community/stylix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    nur = { # Nix User Repository
      url = "github:nix-community/NUR";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    home-manager = {
      url = "github:nix-community/home-manager/master";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    nvf = {
      url = "github:notashelf/nvf";
    };
  };

 outputs = inputs: 
  let
    system = "x86_64-linux";

    # create a list of all directories inside of ./hosts
    # every directory in ./hosts has a config for that machine
    hosts = builtins.filter (x: x != null) (
      inputs.nixpkgs.lib.mapAttrsToList (name: value: if (value == "directory") then name else null) (
        builtins.readDir ./hosts
      )
    );
  in
  {

    nixosConfigurations = builtins.listToAttrs (
      map (
        host: 
        let
          shared = import (./hosts + "/${host}/shared.nix");
        in
        {
          name = host;
          value = inputs.nixpkgs.lib.nixosSystem {
            inherit system;
    
            specialArgs = { # pkgs, lib, config, options, modulesPath
              inherit inputs;
              inherit shared;
            };
    
            modules = [
              # import all system modules 
              (inputs.import-tree ./modules/system)

              # host specific config
              { config.networking.hostName = host; }
              (./hosts + "/${host}")
    
              # integrate nur (pkgs.nur)
              inputs.nur.modules.nixos.default

              # integrate stylix for desktop theming
              inputs.stylix.nixosModules.stylix
    
              # add pkgs overlays
              {
                nixpkgs.overlays = [
                  # add pkgs.stable
                  (final: prev: {
                    stable = import inputs.nixpkgs-stable {
                      inherit (final.stdenv.hostPlatform) system;
                      inherit (final) config;
                    };
                  })
                ];
              }
    
              # integrate home manager
              inputs.home-manager.nixosModules.home-manager
    
              {
                # force home manager to use nixpkgs
                home-manager.useGlobalPkgs = true;
                home-manager.useUserPackages = true;

                # give home manager args
                home-manager.extraSpecialArgs = {
                  inherit inputs;
                  inherit shared;
                };
              }
            ];

          };
        }
      ) hosts
    );
  };

}
