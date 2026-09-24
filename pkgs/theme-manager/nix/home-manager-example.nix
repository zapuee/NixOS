# Add this project as a flake input named `theme-manager`, then import its
# Home Manager module. The module installs the package, starts the watcher, and
# automatically adds generated includes when Niri or Foot are enabled.
{ inputs, ... }:

{
  imports = [
    inputs.theme-manager.homeModules.default
  ];

  programs.theme-manager.enable = true;
}
