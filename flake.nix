{
  description = "Development dependencies of spencerOS";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  # TODO : Understand what self does
  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        #rustVersion = pkgs.rust-bin.stable.latest.default;
        rustVersion = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [ "rust-src" ];
          # targets = [ "x86_64-unknown-none" ];
        };
        #rustVersion = (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
        #rustPlatform = pkgs.makeRustPlatform {
        #  cargo = rustVersion;
        #  rustc = rustVersion;
        #};

        buildInputs = with pkgs; [
          # coreboot-toolchain.i386
          nasm
          qemu
          unixtools.xxd
          rustVersion
          gdb
        ];
        inherit (pkgs) stdenv;
      in
      {
        defaultPackage = stdenv.mkDerivation {
          inherit buildInputs;
          name = "test_boot";
          src = ./.;
          buildPhase = ''
            make
            #nasm -f bin ./src/asm/boot.asm -o boot.bin
            # nasm kernel_entry.asm -f elf -o kernel_entry.o
            #qemu-system-x86_64 -drive file=boot.bin,format=raw,index=0,media=disk
            cargo build
          '';
        };
        devShells.default = pkgs.mkShell {
          inherit buildInputs;

          packages = [ pkgs.bashInteractive ];
          # packages = [ toolchain ];
          nativeBuildInputs = [ ];
          shellHook = ''
            export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${builtins.toString (pkgs.lib.makeLibraryPath buildInputs)}";
          '';
        };
      });
}
