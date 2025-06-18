{
  description = "Ahiru-TPM";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    # Rust toolchains
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # Library for building Cargo projects
    crane.url = "github:ipetkov/crane";
  };

  outputs = {
    nixpkgs,
    flake-utils,
    crane,
    ...
  } @ inputs:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
        fenix = inputs.fenix.packages.${system};

        fenix-shell-profile = fenix.stable;
        fenix-build-toolchain = fenix.stable.minimalToolchain;

        craneLib = (crane.mkLib pkgs).overrideToolchain fenix-build-toolchain;

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
        devShells = {
          default = import ./shell.nix {
            inherit pkgs;
            inherit fenix;
            inherit fenix-shell-profile;
          };
        };

        packages = {
          default = packages.ahiru-tpm;
          ahiru-tpm = craneLib.buildPackage {src = cleanCargoSource ./.;};
        };

        overlays = {
          default = overlays.ahiru-tpm;
          ahiru-tpm = _final: _prev: {ahiru-tpm = packages.ahiru-tpm;};
        };
      }
    );
}
