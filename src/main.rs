fn main() {
    // read env variables that were set in build script
    let uefi_path = env!("UEFI_PATH");
    let bios_path = env!("BIOS_PATH");

    // choose whether to start the UEFI or BIOS image
    let uefi = false;

    let mut cmd = std::process::Command::new("qemu-system-x86_64");
    if uefi {
        //cmd.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
        cmd.arg("-drive")
            .arg(format!("format=raw,file={uefi_path}"));
    } else {
        cmd.arg("-drive")
            .arg(format!("format=raw,file={bios_path}"));
    }
    cmd.args([
        "-device",
        "isa-debug-exit,iobase=0xf4,iosize=0x04",
        "-serial",
        "stdio",
        "-device",
        "virtio-mouse",
        "-device",
        "virtio-keyboard",
    ]);
    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap();
}
