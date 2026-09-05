{ config, lib, inputs, pkgs, ... }:

{
  config = {
    home.file.".mozilla/firefox".source =
      config.lib.file.mkOutOfStoreSymlink
        "${config.home.homeDirectory}/.config/mozilla/firefox";

    home.activation.pywalfoxNoctalia =
      lib.hm.dag.entryAfter [ "linkGeneration" ] ''
        ${inputs.noctalia.packages.${pkgs.system}.default}/bin/noctalia firefox-theme install
      '';

    programs.firefox = {
      enable = true;
      package = pkgs.firefox;
      
      policies = {
	DisableTelemetry = true;
	DisableFirefoxStudies = true;
	DontCheckDefaultBrowser = true;
	DisablePocket = true;

	ExtensionSettings = {
          "{7c7f6dea-3957-4bb9-9eec-2ef2b9e5bcec}" = {
            installation_mode = "force_installed";
            install_url =
              "https://addons.mozilla.org/firefox/downloads/latest/ultimadark/latest.xpi";
          };
        };
      };

      profiles = {
	mah_main = {
	  id = 0;

	  extensions = {
	    packages = with pkgs.nur.repos.rycee.firefox-addons; [
	      ublock-origin
	      new-tab-override
	      vimium
	      pywalfox
	    ];
	  };

	  settings = {
	    "extensions.autoDisableScopes" = 0; #automatically enable extensions
	    "browser.startup.homepage" = "https://www.google.com";   
	    "browser.compactmode.show" = true;
	    "toolkit.legacyUserProfileCustomizations.stylesheets" = true;
	    "browser.tabs.allow_transparent_browser" = true;
	  };
	};
      };
    };

  };
}

