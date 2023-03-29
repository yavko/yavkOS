# yavkOS - A OS that attempts at running WASM modules as userspace programs

## Recommended Development Environment
You need [nix](https://nixos.org) with the `flakes`, and `nix-command` experimental features enabled!
Now wherever in the repo run `nix develop`
This will set up all needed deps to build and develop

## Building
It's as simple as `cargo build`!

## Running
I haven't tested on real hardware, but to virtualize
install QEMU (included in nix dev env), and just
run `cargo run`!

## Architecture (WIP)
This is mostly based on the posts of `blog_os`
