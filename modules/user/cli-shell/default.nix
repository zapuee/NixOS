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
        
      initContent = ''
        bindkey '^P' up-line-or-history
        bindkey '^N' down-line-or-history

        cd() {
          if [[ "$1" =~ '^[0-9]+$' ]]; then
            local path="."
            for ((i=0; i<$1; i++)); do
              path="$path/.."
            done
            builtin cd "$path"
          else
            builtin cd "$@"
          fi
        }
      '';

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
