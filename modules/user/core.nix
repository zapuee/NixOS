{
  config,
  host,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.userSettings.wallpaperServer;
in
{
  options.userSettings.wallpaperServer = {
    enable = lib.mkEnableOption "the local wallpaper HTTP server";

    directory = lib.mkOption {
      type = lib.types.str;
      default = "Pictures/Wallpapers";
      description = "Directory to serve, relative to the user's home directory.";
    };

    port = lib.mkOption {
      type = lib.types.port;
      default = 8080;
      description = "Loopback port used by the wallpaper HTTP server.";
    };
  };

  config = {
    home = {
      inherit (host) stateVersion username;
      homeDirectory = "/home/${host.username}";
    };

    systemd.user.services.wallpaper-server = lib.mkIf cfg.enable {
      Unit.Description = "Local wallpaper server";

      Service = {
        ExecStart = "${lib.getExe pkgs.python3} -m http.server ${toString cfg.port} --bind 127.0.0.1";
        WorkingDirectory = "%h/${cfg.directory}";
        Restart = "on-failure";
      };

      Install.WantedBy = [ "default.target" ];
    };
  };
}
