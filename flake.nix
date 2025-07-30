{
  description = "A simple rust dev environment";

  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/*.tar.gz";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils = {
        url = "github:numtide/flake-utils";
    };
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        rust-overlays = if builtins.pathExists ./rust-toolchain.toml then [
            (import rust-overlay)
            (final: prev: {
                rustToolchain = prev.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
            })
        ] else [
            (final: prev: {
                rustToolchain = with prev; [ 
                    cargo 
                    clippy
                    rustc
                    rustfmt
                    rust-analyzer
                ];
            })
        ];

        overlays = rust-overlays ++ [];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      {
        devShells.default = with pkgs; mkShell {
          buildInputs = [
            openssl
            pkg-config
            rustToolchain
            cargo-deny
            cargo-edit
          ];

          shellHook = ''
          '';
        };
      }
    );
}
