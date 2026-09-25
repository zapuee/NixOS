{
  description = "Zap's modular NixOS and Home Manager configurations";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    nix-flatpak.url = "github:gmodena/nix-flatpak/?ref=latest";

    import-tree.url = "github:denful/import-tree";
    flake-parts.url = "github:hercules-ci/flake-parts";

    noctalia = {
      url = "github:noctalia-dev/noctalia";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    stylix = {
      url = "github:nix-community/stylix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    spicetify-nix = {
      url = "github:Gerg-L/spicetify-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

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
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
      ];

      imports = [
        ./flake/hosts.nix
      ];

      flake.homeModules = {
        default = import ./pkgs/theme-manager/nix/home-manager-module.nix;
        theme-manager = import ./pkgs/theme-manager/nix/home-manager-module.nix;
      };

      perSystem =
        { pkgs, ... }:
        let
          themeManager = pkgs.callPackage ./pkgs/theme-manager/package.nix { };
          hostAppearanceInventory =
            inputs.self.nixosConfigurations.homer.config.home-manager.users.zap.xdg.configFile."theme-manager/appearance-inventory.toml".source;
        in
        {
          packages = {
            default = themeManager;
            theme-manager = themeManager;
          };

          checks = {
            theme-manager = themeManager;

            theme-manager-host-plan =
              pkgs.runCommand "theme-manager-host-plan"
                {
                  nativeBuildInputs = [ themeManager ];
                }
                ''
                  export XDG_CONFIG_HOME="$TMPDIR/config"
                  export XDG_CACHE_HOME="$TMPDIR/cache"
                  export XDG_STATE_HOME="$TMPDIR/state"
                  mkdir -p \
                    "$XDG_CONFIG_HOME/theme-manager" \
                    "$XDG_CACHE_HOME/theme-manager" \
                    "$XDG_STATE_HOME"
                  ln -s ${./pkgs/theme-manager/profiles} "$XDG_CONFIG_HOME/theme-manager/profiles"
                  ln -s ${hostAppearanceInventory} \
                    "$XDG_CONFIG_HOME/theme-manager/appearance-inventory.toml"
                  ln -s \
                    ${./pkgs/theme-manager/crates/theme-manager/tests/fixtures/noctalia-theme.json} \
                    "$XDG_CACHE_HOME/theme-manager/noctalia-palette.json"

                  theme-manager \
                    --config ${./modules/user/theme-manager/config.toml} \
                    check glass --json > "$out"
                  grep -q '"complete": true' "$out"
                  grep -q '"item": "bibata-modern-ice"' "$out"
                  grep -q '"capability": "csd"' "$out"
                '';

            nix-source =
              pkgs.runCommand "nix-source-checks"
                {
                  nativeBuildInputs = with pkgs; [
                    deadnix
                    nixfmt
                    statix
                  ];
                  src = inputs.self;
                }
                ''
                  cp -R "$src" source
                  chmod -R u+w source
                  cd source

                  deadnix --fail .
                  statix check .
                  find . -name '*.nix' -print0 | xargs -0 nixfmt --check

                  touch "$out"
                '';
          };
          formatter = pkgs.nixfmt-tree;

          devShells.default = pkgs.mkShell {
            packages = with pkgs; [
              cargo
              clippy
              rustc
              rust-analyzer
              rustfmt
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
