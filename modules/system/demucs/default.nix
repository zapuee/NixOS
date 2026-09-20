{ lib, config, pkgs, ... }:

{
  options.systemSettings.demucs.enable =
    lib.mkEnableOption "Enable Demucs audio source separation";

  config = lib.mkIf config.systemSettings.demucs.enable {
    environment.systemPackages = with pkgs; [
      demucs-rs
      ffmpeg
    ];
  };
}
