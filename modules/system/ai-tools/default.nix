{
  pkgs,
  config,
  lib,
  ...
}:

{
  options.systemSettings.aiTools.enable = lib.mkEnableOption "AI tools";

  config = lib.mkIf config.systemSettings.aiTools.enable {
    environment.systemPackages = with pkgs; [
      upscayl
      ollama
      comfyui
      codex
    ];
  };
}
