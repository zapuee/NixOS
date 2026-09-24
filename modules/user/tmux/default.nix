{
  pkgs,
  config,
  lib,
  ...
}:

let
  cfg = config.userSettings.tmux;

  projectFiles = lib.filterAttrs (
    name: type: type == "regular" && (lib.hasSuffix ".yaml" name || lib.hasSuffix ".yml" name)
  ) (builtins.readDir ./.);

  projects = lib.mapAttrs' (
    name: _:
    let
      projectName = lib.removeSuffix ".yaml" (lib.removeSuffix ".yml" name);
    in
    lib.nameValuePair ".config/tmuxp/${projectName}.yaml" {
      source = ./. + "/${name}";
    }
  ) projectFiles;
in
{
  options.userSettings.tmux = {
    enable = lib.mkEnableOption "For terminal development";
  };

  config = lib.mkIf cfg.enable {
    home.packages = [
      pkgs.tmuxp
    ];

    home.file = projects;

    programs.tmux = {
      enable = true;
      clock24 = false;
      keyMode = "vi";
      mouse = true;
      prefix = "C-a";

      extraConfig = ''
        set -g status-position top

        set -g pane-border-lines double
        set -g pane-active-border-style 'bold'
      '';

      plugins = with pkgs.tmuxPlugins; [
        sensible
        resurrect
        continuum
        vim-tmux-navigator
        yank
        jump
        minimal-tmux-status
        tmux-which-key
      ];
    };
  };
}
