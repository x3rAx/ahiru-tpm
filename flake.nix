{
  description = "A Rust Flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    cargo2nix = {
      # TODO: Remove `/main` at the end once rust 2024 is supported in release (which is likely 0.12.0)
      url = "github:cargo2nix/cargo2nix/main";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
      inputs.rust-overlay.follows = "rust-overlay";
    };
  };

  outputs = {
    nixpkgs,
    flake-utils,
    rust-overlay,
    cargo2nix,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [
          (import rust-overlay)
          cargo2nix.overlays.default
          (final: prev: {cargo2nix = cargo2nix.packages.${system}.cargo2nix;})
        ];

        pkgs = import nixpkgs {inherit system overlays;};

        rustPkgs = pkgs.rustBuilder.makePackageSet {
          rustVersion = "1.87.0";
          packageFun = import ./Cargo.nix;
        };
      in rec {
        devShells.default = import ./shell.nix {inherit pkgs;};

        packages = {
          default = packages.ahiru-tpm;
          ahiru-tpm = rustPkgs.workspace.ahiru-tpm {};
        };
      }
    );
}
