{ osConfig, config, lib, pkgs, ... }:

let
  cfg = config.userSettings.shell.apps;

  btopPatch = # fix nvidia gpu support
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

    # fuzzy finder
    programs.television = {
      enable = true;
      enableZshIntegration = true;
      channels = {
        zsh-history = {
          metadata = {
            name = "zsh-history";
            description = "Zsh command history";
            requirements = [ "tac" "sed" ];
          };
          source = {
            command = ''tac "''${HISTFILE:-$HOME/.zsh_history}" | sed 's/^: [0-9]*:[0-9]*;//' '';
            output = "{split:\n}";
          };
          preview = {
            command = "echo {}";
          };
        };
      };
    };
    programs.zsh = { # add some tv shortcuts
      initContent = ''
        tvn() {
          local file
          file=$(tv files)
          [[ -n "$file" ]] && nvim -- "$file"
        }

        tcd() {
          local dir
          dir=$(tv dirs)
          [[ -n "$dir" && -d "$dir" ]] && cd -- "$dir"
        }

        tvh() {
          local command
          command=$(tv zsh-history)
          [[ -n "$command" ]] && print -z -- "$command"
        }
      '';
    };

    # file system (yazi)
    programs.yazi = {
      enable = true;
      enableZshIntegration = true;
    };

    home.packages = with pkgs; [
      killall
      trashy
      btopPatch
      coreutils
    ];
  };
}
