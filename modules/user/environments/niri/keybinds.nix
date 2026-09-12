{ config, pkgs, osConfig, lib, ... }:

let
  cfg = osConfig.systemSettings.environment;
in
{
  config = lib.mkIf (cfg == builtins.baseNameOf ./.) {
    wayland.windowManager.niri.settings.binds = {
      # Hotkey overlay
      "Mod+Shift+Slash".show-hotkey-overlay = {};

      # Applications
      "Mod+T" = {
        _props.hotkey-overlay-title =
          "Open a Terminal: ${config.userSettings.terminal}";
        spawn = [ config.userSettings.terminal ];
      };

      "Mod+Shift+S".spawn = [
        "sh"
        "-c"
        ''grim -g "$(slurp)" ~/Pictures/screenshot-$(date +%Y-%m-%d_%H-%M-%S).png''
      ];

      "Mod+Space" = {
        spawn = [ "noctalia" "msg" "panel-toggle" "control-center" "home" ];
      };

      "Alt+Space" = {
        _props.hotkey-overlay-title = "Open Noctalia Launcher";
        spawn = [ "noctalia" "msg" "panel-toggle" "launcher" ];
      };

      "Super+Alt+S" = {
        _props = {
          hotkey-overlay-title = null;
          allow-when-locked = true;
        };
        spawn-sh = "pkill orca || exec orca";
      };

      # Volume
      "XF86AudioRaiseVolume" = {
        _props.allow-when-locked = true;
        spawn-sh = "wpctl set-volume @DEFAULT_AUDIO_SINK@ 0.1+ -l 1.0";
      };

      "XF86AudioLowerVolume" = {
        _props.allow-when-locked = true;
        spawn-sh = "wpctl set-volume @DEFAULT_AUDIO_SINK@ 0.1-";
      };

      "XF86AudioMute" = {
        _props.allow-when-locked = true;
        spawn-sh = "wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle";
      };

      "XF86AudioMicMute" = {
        _props.allow-when-locked = true;
        spawn-sh = "wpctl set-mute @DEFAULT_AUDIO_SOURCE@ toggle";
      };

      # Media
      "XF86AudioPlay" = {
        _props.allow-when-locked = true;
        spawn-sh = "playerctl play-pause";
      };

      "XF86AudioPause" = {
        _props.allow-when-locked = true;
        spawn-sh = "playerctl play-pause";
      };

      "XF86AudioStop" = {
        _props.allow-when-locked = true;
        spawn-sh = "playerctl stop";
      };

      "XF86AudioPrev" = {
        _props.allow-when-locked = true;
        spawn-sh = "playerctl previous";
      };

      "XF86AudioNext" = {
        _props.allow-when-locked = true;
        spawn-sh = "playerctl next";
      };

      # Brightness
      "XF86MonBrightnessUp" = {
        _props.allow-when-locked = true;
        spawn = [
          "brightnessctl"
          "--class=backlight"
          "set"
          "+10%"
        ];
      };

      "XF86MonBrightnessDown" = {
        _props.allow-when-locked = true;
        spawn = [
          "brightnessctl"
          "--class=backlight"
          "set"
          "10%-"
        ];
      };

      # Overview / windows
      "Mod+O" = {
        _props.repeat = false;
        toggle-overview = {};
      };

      "Mod+Q" = {
        _props.repeat = false;
        close-window = {};
      };

      # Focus columns/windows
      "Mod+Left".focus-column-left = {};
      "Mod+Down".focus-window-down = {};
      "Mod+Up".focus-window-up = {};
      "Mod+Right".focus-column-right = {};

      "Mod+H".focus-column-left = {};
      "Mod+J".focus-window-down = {};
      "Mod+K".focus-window-up = {};
      "Mod+L".focus-column-right = {};

      # Move columns/windows
      "Mod+Ctrl+Left".move-column-left = {};
      "Mod+Ctrl+Down".move-window-down = {};
      "Mod+Ctrl+Up".move-window-up = {};
      "Mod+Ctrl+Right".move-column-right = {};

      "Mod+Ctrl+H".move-column-left = {};
      "Mod+Ctrl+J".move-window-down = {};
      "Mod+Ctrl+K".move-window-up = {};
      "Mod+Ctrl+L".move-column-right = {};

      # First/last column
      "Mod+Home".focus-column-first = {};
      "Mod+End".focus-column-last = {};
      "Mod+Ctrl+Home".move-column-to-first = {};
      "Mod+Ctrl+End".move-column-to-last = {};

      # Focus monitor
      "Mod+Shift+Left".focus-monitor-left = {};
      "Mod+Shift+Down".focus-monitor-down = {};
      "Mod+Shift+Up".focus-monitor-up = {};
      "Mod+Shift+Right".focus-monitor-right = {};

      "Mod+Shift+H".focus-monitor-left = {};
      "Mod+Shift+J".focus-monitor-down = {};
      "Mod+Shift+K".focus-monitor-up = {};
      "Mod+Shift+L".focus-monitor-right = {};

      # Move columns to monitor
      "Mod+Shift+Ctrl+Left".move-column-to-monitor-left = {};
      "Mod+Shift+Ctrl+Down".move-column-to-monitor-down = {};
      "Mod+Shift+Ctrl+Up".move-column-to-monitor-up = {};
      "Mod+Shift+Ctrl+Right".move-column-to-monitor-right = {};

      "Mod+Shift+Ctrl+H".move-column-to-monitor-left = {};
      "Mod+Shift+Ctrl+J".move-column-to-monitor-down = {};
      "Mod+Shift+Ctrl+K".move-column-to-monitor-up = {};
      "Mod+Shift+Ctrl+L".move-column-to-monitor-right = {};

      # Workspace navigation
      "Mod+Page_Down".focus-workspace-down = {};
      "Mod+Page_Up".focus-workspace-up = {};
      "Mod+D".focus-workspace-down = {};
      "Mod+U".focus-workspace-up = {};

      "Mod+Ctrl+Page_Down".move-column-to-workspace-down = {};
      "Mod+Ctrl+Page_Up".move-column-to-workspace-up = {};
      "Mod+Ctrl+U".move-column-to-workspace-down = {};
      "Mod+Ctrl+I".move-column-to-workspace-up = {};

      "Mod+Shift+Page_Down".move-workspace-down = {};
      "Mod+Shift+Page_Up".move-workspace-up = {};
      "Mod+Shift+U".move-workspace-down = {};
      "Mod+Shift+I".move-workspace-up = {};

      # Workspace scrolling
      "Mod+WheelScrollDown" = {
        _props.cooldown-ms = 150;
        focus-workspace-down = {};
      };

      "Mod+WheelScrollUp" = {
        _props.cooldown-ms = 150;
        focus-workspace-up = {};
      };

      "Mod+Ctrl+WheelScrollDown" = {
        _props.cooldown-ms = 150;
        move-column-to-workspace-down = {};
      };

      "Mod+Ctrl+WheelScrollUp" = {
        _props.cooldown-ms = 150;
        move-column-to-workspace-up = {};
      };

      # Horizontal scrolling
      "Mod+WheelScrollRight".focus-column-right = {};
      "Mod+WheelScrollLeft".focus-column-left = {};

      "Mod+Ctrl+WheelScrollRight".move-column-right = {};
      "Mod+Ctrl+WheelScrollLeft".move-column-left = {};

      "Mod+Shift+WheelScrollDown".focus-column-right = {};
      "Mod+Shift+WheelScrollUp".focus-column-left = {};

      "Mod+Ctrl+Shift+WheelScrollDown".move-column-right = {};
      "Mod+Ctrl+Shift+WheelScrollUp".move-column-left = {};

      # Workspaces 1-9
      "Mod+1".focus-workspace = 1;
      "Mod+2".focus-workspace = 2;
      "Mod+3".focus-workspace = 3;
      "Mod+4".focus-workspace = 4;
      "Mod+5".focus-workspace = 5;
      "Mod+6".focus-workspace = 6;
      "Mod+7".focus-workspace = 7;
      "Mod+8".focus-workspace = 8;
      "Mod+9".focus-workspace = 9;

      "Mod+Ctrl+1".move-column-to-workspace = 1;
      "Mod+Ctrl+2".move-column-to-workspace = 2;
      "Mod+Ctrl+3".move-column-to-workspace = 3;
      "Mod+Ctrl+4".move-column-to-workspace = 4;
      "Mod+Ctrl+5".move-column-to-workspace = 5;
      "Mod+Ctrl+6".move-column-to-workspace = 6;
      "Mod+Ctrl+7".move-column-to-workspace = 7;
      "Mod+Ctrl+8".move-column-to-workspace = 8;
      "Mod+Ctrl+9".move-column-to-workspace = 9;

      # Consume / expel windows
      "Mod+BracketLeft".consume-or-expel-window-left = {};
      "Mod+BracketRight".consume-or-expel-window-right = {};

      "Mod+Comma".consume-window-into-column = {};
      "Mod+Period".expel-window-from-column = {};

      # Column width / window height
      "Mod+R".switch-preset-column-width = {};
      "Mod+Shift+R".switch-preset-column-width-back = {};

      "Mod+Ctrl+Shift+R".switch-preset-window-height = {};
      "Mod+Ctrl+R".reset-window-height = {};

      # Maximize / fullscreen
      "Mod+F".maximize-column = {};
      "Mod+Shift+F".fullscreen-window = {};
      "Mod+M".maximize-window-to-edges = {};
      "Mod+Ctrl+F".expand-column-to-available-width = {};

      # Center
      "Mod+C".center-column = {};
      "Mod+Ctrl+C".center-visible-columns = {};

      # Width adjustments
      "Mod+Minus".set-column-width = "-10%";
      "Mod+Equal".set-column-width = "+10%";

      # Height adjustments
      "Mod+Shift+Minus".set-window-height = "-10%";
      "Mod+Shift+Equal".set-window-height = "+10%";

      # Floating / tiling
      "Mod+V".toggle-window-floating = {};
      "Mod+Shift+V".switch-focus-between-floating-and-tiling = {};

      # Tabbed columns
      "Mod+W".toggle-column-tabbed-display = {};

      # Screenshots
      "Print".screenshot = {};
      "Ctrl+Print".screenshot-screen = {};
      "Alt+Print".screenshot-window = {};

      # Keyboard shortcuts inhibitor
      "Mod+Escape" = {
        _props.allow-inhibiting = false;
        toggle-keyboard-shortcuts-inhibit = {};
      };

      # Quit
      "Mod+Shift+E".quit = {};
      "Ctrl+Alt+Delete".quit = {};

      # Optional
      # "Mod+Shift+P".power-off-monitors = {};
    };
  };
}
