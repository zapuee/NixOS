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

      profiles = {
        mah_main = {
          id = 0;

	        extensions = {
	          packages = with pkgs.nur.repos.rycee.firefox-addons; [
	            ublock-origin
	            darkreader
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

