{
  description = "ESP32-S3 Rust dev shell (espup + Nix)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            # Official esp-rs toolchain installer
            espup

            # Flashing / runner tools
            espflash
            ldproxy

            # Code generator
            esp-generate

            # Editor support
            rust-analyzer
          ];

          CARGO_BUILD_TARGET = "xtensa-esp32s3-none-elf";
          RUST_BACKTRACE = 1;

          shellHook = ''
            echo "[ESP32-S3 dev shell]"
            if [ -f "$HOME/export-esp.sh" ]; then
              . "$HOME/export-esp.sh"
              echo "Loaded ESP Rust toolchain from $HOME/export-esp.sh"

              # Ensure the esp toolchain's bin directory is first in PATH
              ESP_TOOLCHAIN_BIN="$(rustup which rustc --toolchain esp 2>/dev/null | xargs dirname || true)"
              if [ -n "$ESP_TOOLCHAIN_BIN" ]; then
                export PATH="$ESP_TOOLCHAIN_BIN:$PATH"
                echo "Using esp toolchain rustc from: $ESP_TOOLCHAIN_BIN"
              else
                echo "Warning: could not find esp toolchain bin dir via rustup."
              fi
            else
              echo "No ESP Rust toolchain found."
              echo "Run this once to install it:"
              echo "  espup install"
            fi
          '';
        };
      }
    );
}
