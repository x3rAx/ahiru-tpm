{
  description = "Ahiru-TPM";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    crane.url = "github:ipetkov/crane";
  };

  outputs = {
    nixpkgs,
    flake-utils,
    rust-overlay,
    crane,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [
          (import rust-overlay)
        ];

        pkgs = import nixpkgs {inherit system overlays;};

        getRustToolchain = pkgs: pkgs.rust-bin.stable.latest.default;

        craneLib = (crane.mkLib pkgs).overrideToolchain getRustToolchain;

        cleanCargoSource = src:
          pkgs.lib.cleanSourceWith {
            inherit src;
            filter = path: type: let
              isCargoSource = craneLib.filterCargoSources path type;
              isPestFile = type == "regular" && (builtins.match ".*\.pest" path) != null;
            in
              isCargoSource || isPestFile;
          };
      in rec {
        devShells.default = import ./shell.nix {
          inherit pkgs;
          rust-toolchain = getRustToolchain pkgs;
        };

        packages = {
          default = packages.ahiru-tpm;
          ahiru-tpm = craneLib.buildPackage {src = cleanCargoSource ./.;};
        };
      }
    );
}
