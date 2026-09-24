{
  lib,
  makeWrapper,
  rustPlatform,
}:

rustPlatform.buildRustPackage {
  pname = "theme-manager";
  version = "0.2.0";

  src = lib.fileset.toSource {
    root = ./.;

    fileset = lib.fileset.unions [
      ./Cargo.toml
      ./Cargo.lock
      ./crates
      ./plugins
      ./examples/profiles
      ./examples/plugins
      ./api
      ./integrations/noctalia
      ./nix/default-config.toml
    ];
  };

  cargoLock.lockFile = ./Cargo.lock;

  nativeBuildInputs = [ makeWrapper ];

  cargoBuildFlags = [
    "-p"
    "theme-manager"
  ];

  postInstall = ''
    dataDir="$out/share/theme-manager"
    install -Dm444 nix/default-config.toml "$dataDir/config.toml"
    install -Dm444 examples/profiles/glass.toml "$dataDir/profiles/glass.toml"
    install -Dm444 examples/profiles/paper.toml "$dataDir/profiles/paper.toml"
    cp -R integrations/noctalia "$dataDir/noctalia"
    cp -R api "$dataDir/luau"
    cp -R plugins "$dataDir/plugins"
    install -d "$dataDir/examples"
    cp -R examples/plugins "$dataDir/examples/plugins"

    wrapProgram "$out/bin/theme-manager" \
      --set-default THEME_MANAGER_DATA_DIR "$dataDir"
  '';

  doInstallCheck = true;
  installCheckPhase = ''
    runHook preInstallCheck

    testHome="$(mktemp -d)"
    export HOME="$testHome"
    export XDG_CONFIG_HOME="$testHome/config"
    export XDG_STATE_HOME="$testHome/state"
    export XDG_CACHE_HOME="$testHome/cache"

    "$out/bin/theme-manager" check
    "$out/bin/theme-manager" doctor --json | grep -q '"schema": 1'
    "$out/bin/theme-manager" plugins check
    "$out/bin/theme-manager" apply glass
    test -f "$XDG_CONFIG_HOME/theme-manager/generated/niri.kdl"
    test -f "$XDG_CONFIG_HOME/theme-manager/generated/foot.ini"
    "$out/bin/theme-manager" components list --json | grep -q '"target": "foot"'
    "$out/bin/theme-manager" components disable foot
    test ! -e "$XDG_CONFIG_HOME/theme-manager/generated/foot.ini"
    "$out/bin/theme-manager" components enable foot
    test -f "$XDG_CONFIG_HOME/theme-manager/generated/foot.ini"
    "$out/bin/theme-manager" tui --help | grep -q -- '--no-vim'
    "$out/bin/theme-manager" status --json | grep -q '"active_profile": "glass"'
    test -f "$out/share/theme-manager/luau/theme-manager.d.luau"
    test -f "$out/share/theme-manager/plugins/foot/main.luau"
    test -f "$out/share/theme-manager/plugins/noctalia/main.luau"
    test -f "$out/share/theme-manager/examples/plugins/generic-text/main.luau"

    runHook postInstallCheck
  '';

  passthru.dataDir = "${placeholder "out"}/share/theme-manager";

  meta = {
    description = "Modular desktop theme manager with CLI and TUI frontends";
    mainProgram = "theme-manager";
    license = lib.licenses.mit;
    platforms = lib.platforms.linux;
  };
}
