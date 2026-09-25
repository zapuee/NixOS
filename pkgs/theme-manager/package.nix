{
  lib,
  makeWrapper,
  rustPlatform,
}:

rustPlatform.buildRustPackage {
  pname = "theme-manager";
  version = "0.3.0";

  src = lib.fileset.toSource {
    root = ./.;

    fileset = lib.fileset.unions [
      ./Cargo.toml
      ./Cargo.lock
      ./crates
      ./integrations
      ./profiles
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
    install -d "$dataDir/profiles"
    cp -R profiles/. "$dataDir/profiles/"
    cp -R integrations "$dataDir/integrations"

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
    "$out/bin/theme-manager" theme list --json | grep -q '"name": "glass"'
    "$out/bin/theme-manager" structure list --json | grep -q '"name": "paper"'
    "$out/bin/theme-manager" palette list --json | grep -q '"provider": "static"'
    "$out/bin/theme-manager" apply glass
    test -f "$XDG_CONFIG_HOME/theme-manager/generated/niri.kdl"
    test -f "$XDG_CONFIG_HOME/theme-manager/generated/foot.ini"
    "$out/bin/theme-manager" status --json | grep -q '"active_theme": "glass"'
    test -f "$out/share/theme-manager/integrations/noctalia/palette.json"

    runHook postInstallCheck
  '';

  passthru.dataDir = "${placeholder "out"}/share/theme-manager";

  meta = {
    description = "Modular desktop theme manager with validated application wiring";
    mainProgram = "theme-manager";
    license = lib.licenses.mit;
    platforms = lib.platforms.linux;
  };
}
