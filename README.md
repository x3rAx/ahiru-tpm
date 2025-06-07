<div align="center">
    <img src="https://gitlab.com/x3ro/tpm-rs/-/raw/main/duck_using_tmux_chatgpt.png"
        alt="A duck using tmux (generated with ChatGPT)" width="300">
</div>

# Tmux Plugin Manager RS :crab:

<small>*A drop-in replacement for the famous [Tmux Plugin Manager](https://github.com/tmux-plugins/tpm), written in Rust.*</small>

**tpm-rs** installs, loads and manages **tmux** plugins.

Plugins are loaded in parallel for maximum tmux startup speed.

A curated list of plugins can be found
[here](https://github.com/tmux-plugins/list) (thanks to [Bruno
Sutic](https://github.com/bruno-) for maintaining the list).

## Installation

### Nix Flakes

> *Thanks to [Yazi](https://github.com/yazi-rs/) for the flake template below.*

Below is a basic `flake.nix` showcasing how to add `tpm-rs` to system packages
and through home-manager:

<details>
  <summary>flake.nix</summary>

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";

    home-manager = {
      url = "github:nix-community/home-manager/release-25.05";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    tpm-rs = {
      url = "gitlab:x3ro/tpm-rs";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    nixpkgs,
    home-manager,
    tpm-rs,
    ...
  }: {
    # To install tpm-rs system-wide:
    nixosConfigurations = {
      "nixos" = nixpkgs.lib.nixosSystem {
        modules = [
          ({pkgs, ...}: {
            environment.systemPackages = [tpm-rs.packages.${pkgs.system}.default];
          })
        ];
      };
    };

    # To install it for a specific user using home-manager:
    homeConfigurations = {
      "alice@nixos" = home-manager.lib.homeManagerConfiguration {
        pkgs = nixpkgs.legacyPackages.x86_64-linux;
        modules = [
          ({pkgs, ...}: {
            home.packages = [tpm-rs.packages.${pkgs.system}.default];
          })
        ];
      };
    };
  };
}
```

</details>

Essentially, add `gitlab:x3ro/tpm-rs` to your inputs:

```nix
{
  inputs = {
    tpm-rs = {
      url = "gitlab:x3ro/tpm-rs";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  // ...
}
```

Then add the following to your packages list:

```nix
tpm-rs.packages.${pkgs.system}.default
```

### Build from source

Setup the latest stable Rust toolchain via [rustup](https://rustup.rs/):

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update
```

Clone the repository:

```sh
git clone https://gitlab.com/x3ro/tpm-rs.git
cd tpm-rs
```

Then build `tpm-rs`:

```sh
cargo build --locked
```

Or install it for the current user:

```sh
cargo install --locked --path .
```

## License

[MIT](./LICENSE)
