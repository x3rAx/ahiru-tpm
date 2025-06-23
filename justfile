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


test-gh-action--post-issue:
    #!/usr/bin/env bash
    trap 'rm $env_file' EXIT
    env_file="$(mktemp)"
    echo >"$env_file" '{
        "issue": {
            "title": "Test issue",
            "body": "This is a test issue body",
            "html_url": "https://github.com/test/repo/issues/1"
        },
        "repository": {
            "full_name": "test/repo"
        }
    }'
    # --secret-file \#__DEV__/act.secrets 
    act issues \
        -e "$env_file" \
        --env DRY_RUN=true


# Install crate as local package
install:
    cargo install --path . --locked
