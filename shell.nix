{
  pkgs ? import <nixpkgs> {},
  fenix ? import (fetchTarball "https://github.com/nix-community/fenix/archive/main.tar.gz") {},
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
