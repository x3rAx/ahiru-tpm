let
  inherit (builtins) currentSystem fromJSON readFile;

  getFlake = name: let
    flake = (fromJSON (readFile ./flake.lock)).nodes.${name}.locked;
  in {
    inherit (flake) rev;
    outPath = fetchTarball {
      url = "https://github.com/${flake.owner}/${flake.repo}/archive/${flake.rev}.tar.gz";
      sha256 = flake.narHash;
    };
  };
in
  {
    system ? currentSystem,
    pkgs ? import (getFlake "nixpkgs") {localSystem = {inherit system;};},
    fenix ? import (getFlake "fenix") {},
    fenix-shell-profile ? fenix.stable,
  }:
    pkgs.mkShell {
      name = "nix-shell";

      packages = with pkgs; [
        bashInteractive

        (fenix-shell-profile.withComponents [
          "cargo"
          "rust-src"
          "rust-analyzer"
          "clippy"
        ])
        bacon # CLI test runner
        cargo-watch

        #openssl.dev
        #pkgconfig # Required to find openssl
        #lldb # Install lldb with `lldb-dap` (aka `lldb-vscode`)
      ];
    }
