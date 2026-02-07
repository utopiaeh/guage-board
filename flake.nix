{
  description = "ESP32-S3 Rust dev shell";

  inputs = {
    # You can pin this to a specific revision if you want full reproducibility
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    flake-utils.url = "github:numtide/flake-utils";

    # Rust toolchains from nix-community
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
        pkgs = import nixpkgs {
          inherit system;
        };

        # Use rust-toolchain.toml to define the toolchain (including target support if you like)
        rustToolchain = fenix.packages.${system}.fromToolchainFile {
          file = ./rust-toolchain.toml;
          # Replace this with the correct hash after the first `nix develop` run
          sha256 = "0000000000000000000000000000000000000000000000000000";
        };

        # ESP32 tools are in `pkgs.esp-idf` set in some channels, or as separate packages in others.
        # With nixpkgs-unstable you'll typically have `espflash` and `ldproxy` directly in pkgs.
        espflash = pkgs.espflash;
        ldproxy = pkgs.ldproxy;
      in
      {
        devShells.default = pkgs.mkShell {
          # This gives you `cargo`, `rustc`, etc. from fenix plus ESP tooling
          packages = [
            rustToolchain
            espflash
            ldproxy
          ];

          # Recommended environment for ESP32-S3 development
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";

          # Default target – adjust if you’re using a custom one
          CARGO_BUILD_TARGET = "xtensa-esp32s3-none-elf";

          # Helpful for backtraces and logging when you run host tools
          RUST_BACKTRACE = 1;

          # If you want cargo-espflash via cargo install (inside the toolchain),
          # you can also add a shellHook:
          shellHook = ''
            echo "ESP32-S3 dev shell"
            echo "  target: $CARGO_BUILD_TARGET"
          '';
        };
      }
    );
}
