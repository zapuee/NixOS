{ pkgs, config, lib, ... }:

{
  options.systemSettings.ai.enable = lib.mkEnableOption "ai stuff";
  
  config = {
    environment.systemPackages = with pkgs; [
      upscayl
      ollama
      comfyui
      codex
    ];
  };
}
