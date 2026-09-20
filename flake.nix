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

      perSystem = { pkgs, ... }: {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            rustc
            rust-analyzer
            pkg-config
            clang
            lld
      
            vulkan-loader
            vulkan-tools
      
            wayland
            libxkbcommon
            libx11
            libxcursor
            libxi
            libxrandr
            alsa-lib
            udev
            xkeyboard_config
          ];
      
          LD_LIBRARY_PATH =
            pkgs.lib.makeLibraryPath [
              pkgs.vulkan-loader
              pkgs.wayland
              pkgs.libxkbcommon
              pkgs.libx11
              pkgs.libxcursor
              pkgs.libxi
              pkgs.libxrandr
              pkgs.alsa-lib
              pkgs.udev
            ]
            + ":/run/opengl-driver/lib";

          CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER = "clang";
          RUSTFLAGS = "-C link-arg=-fuse-ld=lld";
          XKB_CONFIG_ROOT = "${pkgs.xkeyboard_config}/etc/X11/xkb";
        };
      };

    };
}
