{ config, lib, pkgs, ... }:

let
  cfg = config.userSettings.shell;
in
{
  options = {
    userSettings.shell = {
      enable = lib.mkEnableOption "Enable some zsh configs and utilities";
    };
  };

  config = lib.mkIf cfg.enable {
    programs.zsh = {
      enable = true;
      enableCompletion = true;
      defaultKeymap = "viins";
      autosuggestion.enable = true;
        
      antidote = {
        enable = true;
        plugins = [
          "zsh-users/zsh-autosuggestions"
          "zsh-users/zsh-syntax-highlighting"
          "zsh-users/zsh-history-substring-search"
        ];
      };

    };
  };
}
