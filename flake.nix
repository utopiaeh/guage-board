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

        # Use fenix's "complete" toolchain for the desired channel
        rustToolchain = fenix.packages.${system}.complete.toolchain;
      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            rustToolchain
            pkgs.espflash
            pkgs.ldproxy
          ];

          # Set the ESP32-S3 target as default
          CARGO_BUILD_TARGET = "xtensa-esp32s3-none-elf";
          RUST_BACKTRACE = 1;

          shellHook = ''
            echo "ESP32-S3 dev shell (fenix complete)"
            echo "  target: $CARGO_BUILD_TARGET"
          '';
        };
      }
    );
}
