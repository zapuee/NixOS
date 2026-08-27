{ inputs, shared, config, ... }:

{
  imports = [
    ./configuration.nix
    ./hardware-configuration.nix
  ];

  config = {
    home-manager.users.${shared.username} = {
      imports = [
        ./home.nix
        (inputs.import-tree ../../modules/user)
      ];
    };
  };
}
