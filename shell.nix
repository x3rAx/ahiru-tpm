{
  pkgs ? import <nixpkgs> {},
  rust-toolchain ? pkgs.rust-bin.stable.latest.default,
}:
pkgs.mkShell {
  name = "nix-shell";

  packages = with pkgs; [
    bashInteractive

    (rust-toolchain.override {
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
    #lldb # Install lldb with `lldb-dap` (aka `lldb-vscode`)
  ];
}
