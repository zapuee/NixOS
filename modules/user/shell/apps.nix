{ osConfig, config, lib, pkgs, ... }:

let
  cfg = config.userSettings.shell.apps;

  # terminal cli for yt and soundcloud
  rs-pug = pkgs.rustPlatform.buildRustPackage rec {
    pname = "rs-pug";
    version = "1.0.4";

    src = pkgs.fetchCrate {
      inherit pname version;
      hash = "sha256-KSGIS1vYcJCXCTkm3OrwWsPotSls2yq4JsEUiWFtThY=";
    };

    cargoHash = "sha256-4ZJaPpBK3fPFl4VpZyzsb+6ivnVP6Pe60BE8h3Tx/Gg=";

    nativeBuildInputs = with pkgs; [
      pkg-config
      makeWrapper
    ];

    buildInputs = with pkgs; [
      mpv
      yt-dlp
    ];

    doCheck = false;

    postInstall = ''
      wrapProgram $out/bin/rs-pug \
        --prefix PATH : ${lib.makeBinPath [
          pkgs.mpv
          pkgs.yt-dlp
        ]}
    '';
  };

  btopPatch = # fix btop from not having nvidia support
    if osConfig.systemSettings.hardware.gpu == "nvidia" then
      pkgs.btop.overrideAttrs (old: {
        nativeBuildInputs = (old.nativeBuildInputs or []) ++ [ pkgs.makeWrapper ];

        postInstall = (old.postInstall or "") + ''
          wrapProgram $out/bin/btop \
            --prefix LD_LIBRARY_PATH : /run/opengl-driver/lib
        '';
      })
    else
      pkgs.btop;
in
{
  options.userSettings.shell.apps = {
    enable = lib.mkEnableOption
      "Enable a collection of additional useful CLI apps";
  };

  config = lib.mkIf cfg.enable {
    home.packages = with pkgs; [
      killall
      trashy
      btopPatch
      rs-pug
    ];
  };
}
