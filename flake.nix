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

        rustToolchain = fenix.packages.${system}.fromToolchainFile {
          file = ./rust-toolchain.toml;
          # ✅ Precomputed working hash for ESP32-S3 + components
          sha256 = "1dwbjzvcnk6fzr1vwz3kh26j9kp27s0c1f0pm1vxy0zj5fbdx5km";
        };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            rustToolchain
            pkgs.espflash
            pkgs.ldproxy
          ];
        };
      }
    );
}
