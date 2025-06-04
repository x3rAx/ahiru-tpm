{pkgs ? import <nixpkgs> {}}:
pkgs.mkShell {
  name = "nix-shell";

  packages = with pkgs; [
    bashInteractive

    (rust-bin.stable.latest.default.override {
      extensions = [
        "rust-src"
        "rust-analyzer"
        "clippy"
      ];
    })
    bacon # CLI test runner
    cargo-watch

    #openssl.dev
    #pkgconfig # Required to find openssl
  ];
}

