set positional-arguments

[private]
@list:
    just --list

[private]
ensure-cargo-bump:

[private]
[confirm("❓ `cargo-bump` is not install. Install it now via `cargo install cargo-bump`? [y/N]:")]
install-cargo-bump:
    cargo install cargo-bump

# Bump version and run `cargo2nix`. Set `version` to `major`, `minor`, `patch` or a specific version string.
bump version: ensure-cargo-bump
    # Ensure cargo-bump is installed
    command -v cargo-bump >/dev/null 2>&1 || just install-cargo-bump;
    # Bump version
    cargo bump '{{version}}'
    # Run cargo2nix
    cargo2nix --overwrite
