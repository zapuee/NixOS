{
  host,
  lib,
  config,
  ...
}:

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

    virtualisation.vmVariant = {
      services.spice-vdagentd.enable = true;
      services.qemuGuest.enable = true;

      virtualisation = {
        memorySize = 8192;
        diskSize = 40 * 1024;
        cores = 6;
        qemu.options = [
          "-device virtio-vga-gl"
          "-display gtk,gl=on"

          "-chardev qemu-vdagent,id=vdagent,name=vdagent,clipboard=on"
          "-device virtio-serial"
          "-device virtserialport,chardev=vdagent,name=com.redhat.spice.0"
        ];
      };

      users.users.${host.username}.initialPassword = "test";
    };
  };
}
