{
  description = "Tryna learn flakes";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    nixpkgs-stable.url = "nixpkgs/nixos-25.11";
    nix-flatpak.url = "github:gmodena/nix-flatpak/?ref=latest";

    import-tree.url = "github:denful/import-tree";
    flake-parts.url = "github:hercules-ci/flake-parts";

    noctalia.url = "github:noctalia-dev/noctalia";
    spicetify-nix.url = "github:Gerg-L/spicetify-nix";

    nur = {
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

  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
      ];

      imports = [
        ./flake/hosts.nix
      ];
    };
}
