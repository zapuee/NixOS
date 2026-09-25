{
  config,
  inputs,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.userSettings.themeManager;
  runtimeConfig = builtins.fromTOML (builtins.readFile ./config.toml);
  bundleCursors = map (bundle: bundle.cursor or null) (
    builtins.attrValues runtimeConfig.theme_bundles
  );
  selectedCursor = builtins.head bundleCursors;
  cursorLibraries = {
    stylix-cursors.bibata-modern-ice = {
      package = pkgs.bibata-cursors;
      name = "Bibata-Modern-Ice";
      size = 24;
    };
  };
  cursorItem =
    cursorLibraries.${selectedCursor.library}.${selectedCursor.item}
      or (throw "unknown cursor selection ${selectedCursor.library}/${selectedCursor.item}");
  appearanceInventory = (pkgs.formats.toml { }).generate "theme-manager-appearance-inventory.toml" {
    schema = 1;
    libraries.${selectedCursor.library} = {
      kind = "cursor";
      source = "stylix.cursor";
      revision = inputs.stylix.rev;
      installation_cadence = "rebuild";
      items.${selectedCursor.item} = {
        installed = true;
        effective = true;
        selection_cadence = "session";
        required_consumers = [
          "home-pointer"
          "gtk"
          "x11"
        ];
        consumers = {
          home-pointer = "stylix.cursor";
          gtk = "stylix.targets.gtk.cursor";
          x11 = "stylix.targets.x11.cursor";
        };
        values = {
          inherit (cursorItem) name;
          package = "${cursorItem.package}";
          size = toString cursorItem.size;
        };
      };
    };
    claims = [
      {
        application = "appearance";
        capability = "cursor_installation";
        owner = "stylix:cursor";
        source = "stylix-cursors/${selectedCursor.item}";
        cadence = "rebuild";
        boundary = "stylix.cursor.package";
      }
      {
        application = "appearance";
        capability = "cursor_selection";
        owner = "stylix:cursor";
        source = "stylix-cursors/${selectedCursor.item}";
        cadence = "session";
        boundary = "home.pointerCursor";
      }
      {
        application = "foot";
        capability = "font";
        owner = "local-nix:foot";
        source = "pkgs.nerd-fonts.lilex";
        cadence = "rebuild";
        boundary = "foot.main.font";
      }
      {
        application = "foot";
        capability = "padding";
        owner = "local-nix:foot";
        source = "programs.foot.settings.main.pad";
        cadence = "rebuild";
        boundary = "foot.main.pad";
      }
      {
        application = "foot";
        capability = "csd";
        owner = "local-nix:foot";
        source = "programs.foot.settings.csd";
        cadence = "rebuild";
        boundary = "foot.csd";
      }
      {
        application = "foot";
        capability = "scrollback";
        owner = "local-nix:foot";
        source = "programs.foot.settings.scrollback.lines";
        cadence = "rebuild";
        boundary = "foot.scrollback";
      }
      {
        application = "firefox";
        capability = "native_messaging";
        owner = "local-nix:firefox";
        source = "noctalia firefox-theme install";
        cadence = "rebuild";
        boundary = "firefox.native-messaging.pywalfox";
      }
      {
        application = "firefox";
        capability = "application_structure";
        owner = "local-nix:firefox";
        source = "browser.tabs.allow_transparent_browser";
        cadence = "rebuild";
        boundary = "firefox.profile.preferences.transparency";
      }
      {
        application = "starship";
        capability = "application_structure";
        owner = "local-nix:starship";
        source = "programs.starship";
        cadence = "rebuild";
        boundary = "starship.prompt.defaults";
      }
    ];
  };
  catalogDirectory = ../../../pkgs/theme-manager/profiles;
  profiles =
    lib.mapAttrs'
      (file: _type: lib.nameValuePair (lib.removeSuffix ".toml" file) (catalogDirectory + "/${file}"))
      (
        lib.filterAttrs (file: type: type == "regular" && lib.hasSuffix ".toml" file) (
          builtins.readDir catalogDirectory
        )
      );
in
{
  imports = [
    inputs.stylix.homeModules.stylix
    ../../../pkgs/theme-manager/nix/home-manager-module.nix
  ];

  options.userSettings.themeManager.enable = lib.mkEnableOption "Theme Manager";

  config = lib.mkIf cfg.enable {
    assertions = [
      {
        assertion = selectedCursor != null && lib.all (cursor: cursor == selectedCursor) bundleCursors;
        message = "all runtime Theme Bundles must select the one effective rebuild-owned cursor";
      }
    ];

    stylix = {
      enable = true;
      autoEnable = false;
      # Stylix requires a palette even when only cursor subtargets are enabled.
      # No color target consumes this internal scheme.
      base16Scheme = "${config.stylix.inputs.tinted-schemes}/base16/catppuccin-macchiato.yaml";
      polarity = "dark";
      cursor = cursorItem;
      targets = {
        gtk.cursor.enable = true;
        x11.cursor.enable = true;
      };
    };

    xdg.configFile."theme-manager/appearance-inventory.toml".source = appearanceInventory;

    programs.theme-manager = {
      enable = true;
      inherit profiles;
      configFile = ./config.toml;
    };
  };
}
