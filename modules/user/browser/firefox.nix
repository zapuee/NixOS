{ pkgs, ... }:

{
  config = {
    programs.firefox = {
      enable = true;
      package = pkgs.firefox;
      #      nativeMessagingHosts = [ 
      #        pkgs.firefoxpwa
      #	      pkgs.kdePackages.plasma-browser-integration
      #      ];
      
      policies = {
	DisableTelemtry = true;
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
	      #darkreader
	      vimium
	    ];
	  };

	  settings = {
	    "extensions.autoDisableScopes" = 0; #automatically enable extensions
	    "browser.startup.homepage" = "https://www.google.com";   
	  };
	};
      };
    };

  };
}

