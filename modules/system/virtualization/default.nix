{ shared, lib, config, ... }:

# you might have to run rm -f Host_Name_Here.qcow2
# to delete old vm disk

let
  cfg = config.systemSettings.virtualization;
in
{
  options.systemSettings.virtualization = {
    enable = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Enable virtualization";
    };
  };

  config = lib.mkIf cfg.enable {
    virtualisation.libvirtd.enable = true;
  
    services.spice-vdagentd.enable = true;
    services.qemuGuest.enable = true;

    virtualisation.vmVariant = {
      virtualisation = {
        memorySize = 8192;
        diskSize = 40 * 1024; # 40 gibs lol
        cores = 6;
        qemu.options = [
          "-device virtio-vga-gl"
          "-display gtk,gl=on"

          "-chardev qemu-vdagent,id=vdagent,name=vdagent,clipboard=on"
          "-device virtio-serial"
          "-device virtserialport,chardev=vdagent,name=com.redhat.spice.0"
        ];
      };

      users.users.${shared.username}.initialPassword = "test";
    };
  };
}
