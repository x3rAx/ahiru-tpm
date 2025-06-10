<div align="center">
    <img src=".res/duck_using_tmux_chatgpt.png"
        alt="A duck using tmux (generated with ChatGPT)" width="300">
</div>

# Tmux Plugin Manager RS :crab:

<small>_A drop-in replacement for the famous [Tmux Plugin Manager](https://github.com/tmux-plugins/tpm), written in Rust._</small>

**tpm-rs** installs, loads and manages **tmux** plugins.

Plugins are loaded in parallel for maximum tmux startup speed (see
[Benchmark](#benchmark)).

A curated list of plugins can be found
[here](https://github.com/tmux-plugins/list) (thanks to [Bruno
Sutic](https://github.com/bruno-) for maintaining the list).

<!-- toc -->

- [Installation](#installation)
    * [Nix Flakes](#nix-flakes)
    * [Build from source](#build-from-source)
- [Usage](#usage)
    * [Installing Plugins](#installing-plugins)
        + [Plugin Spec](#plugin-spec)
    * [Updating Plugins](#updating-plugins)
    * [Uninstalling Plugins](#uninstalling-plugins)
    * [Settings](#settings)
        + [Disable Parallel Mode](#disable-parallel-mode)
    * [Key Bindings](#key-bindings)
- [Benchmark](#benchmark)
    * [2x Faster Plugins Installation](#2x-faster-plugins-installation)
    * [5x Faster Plugins Loading](#5x-faster-plugins-loading)
- [License](#license)

<!-- tocstop -->

## Installation

To let **tpm-rs** manage your tmux plugins, add the following at the end of
your tmux config (either in `~/.tmux.conf` or
`${XDG_CONFIG_HOME}/tmux/tmux.conf`):

> [!Tip]
>
> If you want tmux to start up even faster, change the last line to
>
> ```tmux
> tmux run -b 'tpm init'
> ```
>
> The `-b` runs the init command in the background. This means that `tmux` will
> not wait until `tpm` has initialized but it might cause a bit of flickering
> until all plugins (e.g. themes) are loaded.

```tmux
# List of plugins (install by running `tpm install`)
    set -g @plugin 'tmux-plugins/tmux-sensible'
    set -g @plugin 'x3rAx/tmux-yank#yank-action-mouse'
    set -g @plugin 'catppuccin/tmux'

    # Other examples:
    # set -g @plugin 'github_username/plugin_name'
    # set -g @plugin 'github_username/plugin_name#branch'

# Initialize TMUX plugin manager (keep this line at the very bottom of tmux.conf)
run 'tpm init'
```

Then proceed with one of the following install methods to get `tpm-rs`
installed on your system:

### Nix Flakes

> _Thanks to [Yazi](https://github.com/yazi-rs/) for the flake template below._

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

## Usage

### Installing Plugins

Plugins can be installed by adding them to to the tmux config and running

```sh
tpm install
```

No need to reload tmux config first.

To add a plugin, add the following line to your tmux config:

```tmux
set -g @plugin '<plugin-spec>'
```

Where `<plugin-spec>` is described below:

#### Plugin Spec

Currently plugins can only be installed from GitHub repositories. The
following format is used for the `<plugin-spec>`:

```tmux
github_username/plugin_name
github_username/plugin_name#branch
```

### Updating Plugins

To update plugins run:

```sh
tpm update
```

### Uninstalling Plugins

To uninstall plugins, first remove them from your tmux config. Then, to clean
up downloaded plugins, run:

```sh
tpm clean
```

### Settings

#### Disable Parallel Mode

Sometimes you might want to disable parallel mode, for example to debug a
problem with a plugin or if parallel loading leads to problems for whatever
reason.

This can be done by setting the following in the tmux config:

```tmux
set -g @tpm_parallel 'false'
```

### Key Bindings

Although the default keybindings from the original TPM work here as well, it is
discouraged and we recommend to use these more mnemonic key bindings:

| Key bindings         | Description                    |
| -------------------- | ------------------------------ |
| `prefix` + `alt + I` | **I**nstall and reload plugins |
| `prefix` + `alt + U` | **U**pdate and reload plugins  |
| `prefix` + `alt + C` | **C**lean plugins              |

To change the key bindings add the following to your tmux config:

```tmux
set -g @tpm-bind-install 'M-I'
set -g @tpm-bind-update 'M-U'
set -g @tpm-bind-clean 'M-C'
```

> [!Note]
> As soon as one of the keymaps is changed, the respective "tpm-legacy" binding
> will be disabled and only the one defined by `@tpm-bind-*` will be used.

## Benchmark

In this section we compare the perforemance of the original `tpm` written in
bash and this `tpm-rs` using parallel processing written in rust.

Of course, the loading speed is heavily dependent on the plugins you have,
other processes running on the system and the hardware you are running on.
Therefore the times in the benchmarks are not hard facts but they instead show
a trend.

I closed most of my user apps during benchmarks so that the benchmark runs
without any interruptions and the CPU is not temporarily overloaded by other
programs.

As hardware didn't change between benchmark runs, it's less important but
here's my `inxi` output for completeness:

```
 ❯ inxi
CPU: 14-core (6-mt/8-st) 13th Gen Intel Core i5-13600KF (-MST AMCP-)
speed/min/max: 1100/800/5100:3900 MHz Kernel: 6.12.31 x86_64 Up: 6d 6h 35m
Mem: 7.23/31.17 GiB (23.2%) Storage: 5.97 TiB (59.9% used) Procs: 547
Shell: nu inxi: 3.3.38
```

To make it a bit more reproducible, here's the list of plugins the benchmarks
were run with:

```tmux
set -g @plugin 'tmux-plugins/tpm'
set -g @plugin 'tmux-plugins/tmux-sensible'
set -g @plugin 'christoomey/vim-tmux-navigator'
set -g @plugin 'catppuccin/tmux'
set -g @plugin 'noscript/tmux-mighty-scroll'
```

### 2x Faster Plugins Installation

```sh
hyperfine --warmup 5 --runs 25 --shell nu --prepare 'rm -rf ~/.local/share/tmux/plugins' -n 'tpm-rs' 'tpm install' --prepare 'ls ~/.config/tmux/plugins/ | where ($it.name | path basename) != tpm | each {rm -rf $in.name}' -n 'tpm' 'TMUX_PLUGIN_MANAGER_PATH=($env.HOME)/.config/tmux/plugins/ ~/.config/tmux/plugins/tpm/bin/install_plugins'
```

![Benchmark installing plugins with `tpm-rs` vs. `tpm`. `tpm-rs` ran 2.54 times faster than `tpm`.](.res/hyperfine-tpm-install.png)

### 5x Faster Plugins Loading

```sh
hyperfine --warmup 10 --runs 100 --shell nu -n 'tpm-rs' 'tpm load' -n 'tpm' 'TMUX_PLUGIN_MANAGER_PATH=($env.HOME)/.config/tmux/plugins/ ~/.config/tmux/plugins/tpm/scripts/source_plugins.sh'
```

![Benchmark loading plugins with `tpm-rs` vs. `tpm`. `tpm-rs` ran 5.62 times faster than `tpm`.](.res/hyperfine-tpm-load.png)

## License

[MIT](./LICENSE)
