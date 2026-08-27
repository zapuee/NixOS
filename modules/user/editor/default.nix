{ lib, config, inputs, ... }:

{
  
  options = {
    userSettings.editor = lib.mkOption {
      default = "nvf";
      description = "Code editor";
      type = lib.types.enum [ "nvf" ];
    };
  };

  # conditionals defined within each editor module
}
