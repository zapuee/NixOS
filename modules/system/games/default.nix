{
  pkgs,
  lib,
  config,
  ...
}:

{
  options.systemSettings.games = {
    enable = lib.mkEnableOption "all games";
  };

  config = lib.mkIf config.systemSettings.games.enable {
    programs.steam.enable = true;

    environment.systemPackages = with pkgs; [
      heroic
      prismlauncher
      lutris
      mangohud
      gamescope
      gamemode
      vulkan-tools
      protonup-qt
    ];
    environment.sessionVariables = {
      MANGOHUD = "1";
      MANGOHUD_CONFIG = "fps,frametime,gpu_stats,cpu_stats";
    };
  };
}
