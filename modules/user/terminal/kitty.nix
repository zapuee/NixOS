{
  config,
  lib,
  pkgs,
  ...
}:

{
  config = lib.mkIf (config.userSettings.terminal == "kitty") {
    home.packages = [
      pkgs.kitty
    ];

    programs.kitty = {
      enable = true;
      settings = {
        background_opacity = lib.mkForce "0.85";
        modify_font = "cell_width 90%";
        confirm_os_window_close = 0;
      };
      keybindings = {
        "ctrl+equal" = "change_font_size all +2.0";
        "ctrl+minus" = "change_font_size all -2.0";
      };
    };
  };
}
