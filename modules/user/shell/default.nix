{
  config,
  lib,
  ...
}:
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
        zmodload -i zsh/complist

        # Make Tab enter an actual selectable completion menu
        setopt MENU_COMPLETE
        zstyle ':completion:*' menu select

        # Only active while that selectable menu is open
        bindkey -M menuselect '^H' backward-char
        bindkey -M menuselect '^L' forward-char
        bindkey -M menuselect '^K' reverse-menu-complete
        bindkey -M menuselect '^J' menu-complete

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
