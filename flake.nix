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
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    crane,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            self.overlays.${system}.default
            (import rust-overlay)
          ];
        };

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
      in {
        devShells.default = import ./shell.nix {
          inherit pkgs;
          rust-toolchain = getRustToolchain pkgs;
        };

        packages = {
          default = pkgs.ahiru-tpm;
          ahiru-tpm = pkgs.ahiru-tpm;
        };

        overlays.default = final: prev: {
          ahiru-tpm = craneLib.buildPackage {src = cleanCargoSource ./.;};
        };
      }
    );
}
