{
  description = "Development dependencies of spencerOS";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  # TODO : Understand what self does
  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system ; };
        buildInputs = with pkgs; [
          # coreboot-toolchain.i386
          nasm
          qemu
          unixtools.xxd
          gdb
        ];
        inherit (pkgs) stdenv;
      in
      {
        defaultPackage = stdenv.mkDerivation {
          inherit buildInputs;
          name = "test_boot";
          src = ./.;
          # TODO:
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
