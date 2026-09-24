{ host, pkgs, ... }:

{
  programs.zsh.enable = true;

  users.users.${host.username} = {
    isNormalUser = true;
    description = host.username;
    shell = pkgs.zsh;
    extraGroups = [
      "wheel"
      "networkmanager"
      "audio"
      "video"
      "render"
      "input"
      "storage"
    ];
  };

  system.stateVersion = host.stateVersion;
}
