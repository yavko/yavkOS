{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";
    bootimage-src.url = "github:/rust-osdev/bootimage";
    bootimage-src.flake = false;
  };

  outputs = {
    self,
    rust-overlay,
    nixpkgs,
    bootimage-src,
  }: let
    overlays = [(import rust-overlay)];
    pkgs = import nixpkgs {
      system = "x86_64-linux";
      inherit overlays;
    };
    lockfile = bootimage-src + "/Cargo.lock";
    bootimage = pkgs.rustPlatform.buildRustPackage {
      name = "bootloader";
      pver = bootimage-src.rev;
      src = bootimage-src;
      cargoLock.lockFile = lockfile;
      cargoDeps = pkgs.rustPlatform.importCargoLock {lockFile = lockfile;};
    };
  in {
    devShell.x86_64-linux = pkgs.mkShell {
      buildInputs = [
        (pkgs.rust-bin.selectLatestNightlyWith (toolchain:
          toolchain.default.override {
            extensions = ["rust-src" "llvm-tools"];
            targets = ["x86_64-unknown-none" "x86_64-unknown-linux-gnu"];
          }))
        pkgs.rust-analyzer
        pkgs.rustfmt
        pkgs.qemu
        bootimage
      ];
    };
  };
}
