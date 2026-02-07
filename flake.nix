{
  description = "ESP32-S3 Rust dev shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix.url = "github:nix-community/fenix";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      fenix,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };

        # Full Rust toolchain from fenix
        rustToolchain = fenix.packages.${system}.complete.toolchain;
      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            rustToolchain

            # ESP tools from nixpkgs
            pkgs.espflash
            pkgs.ldproxy
            pkgs.esp-generate
            pkgs.espup
            # not available installed via cargo install globally for now
            # pkgs.esp-config
          ];

          CARGO_BUILD_TARGET = "xtensa-esp32s3-none-elf";
          RUST_BACKTRACE = 1;
        };
      }
    );
}
