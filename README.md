<div align="center">
    <figure>
        <img src=".res/duck_using_tmux_chatgpt.png"
            alt="A duck using tmux (generated with ChatGPT)" width="300">
        <figcaption>
            <i>
                Meet <b>Ahiru</b> 🦆<br />
                She's happy to manage your <b>tmux</b> plugins for you.<br />
            </i>
        </figcaption>
    </figure>
</div>

# Ahiru-TPM 🦆

> _A drop-in replacement for the famous [Tmux Plugin Manager](https://github.com/tmux-plugins/tpm), written in Rust._

**Ahiru-TPM** installs, loads and manages **tmux** plugins.

Plugins are loaded in parallel for maximum tmux startup speed (see
[Benchmark](#benchmark)).

A curated list of plugins can be found
[here](https://github.com/tmux-plugins/list). (Thanks to [Bruno
Sutic](https://github.com/bruno-) for maintaining the list)

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

To let **Ahiru-TPM** manage your tmux plugins, add the following at the end of
your tmux config (either in `~/.tmux.conf` or
`${XDG_CONFIG_HOME}/tmux/tmux.conf`):

> [!Tip]
>
> If you want tmux to start up even faster, change the last line to
>
> ```tmux
> tmux run -b 'ahiru-tpm init'
> ```
>
> The `-b` runs the init command in the background. This means that `tmux` will
> not wait until `ahiru-tpm` has initialized but it might cause a bit of
> flickering until all plugins (e.g. themes) are loaded.

```tmux
# List of plugins (install by running `ahiru-tpm install`)
    set -g @plugin 'tmux-plugins/tmux-sensible'
    set -g @plugin 'x3rAx/tmux-yank#yank-action-mouse'
    set -g @plugin 'catppuccin/tmux'

    # Other examples:
    # set -g @plugin 'github_username/plugin_name'
    # set -g @plugin 'github_username/plugin_name#branch'

# Initialize TMUX plugin manager (keep this line at the very bottom of tmux.conf)
run 'ahiru-tpm init'
```

Then proceed with one of the following install methods to get **Ahiru-TPM**
installed on your system:

### Nix Flakes

_(Thanks to [Yazi](https://github.com/yazi-rs/) for the flake template below.)_

Below is a basic `flake.nix` showcasing how to add **Ahiru-TPM** to system packages
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

    ahiru-tpm = {
      # Use the GitHub mirror here because using the Codeberg url fails for some reason
      url = "github:x3ro/ahiru-tpm";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    nixpkgs,
    home-manager,
    ahiru-tpm,
    ...
  }: {
    # To install ahiru-tpm system-wide:
    nixosConfigurations = {
      "nixos" = nixpkgs.lib.nixosSystem {
        modules = [
          ({pkgs, ...}: {
            environment.systemPackages = [ahiru-tpm.packages.${pkgs.system}.default];
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
            home.packages = [ahiru-tpm.packages.${pkgs.system}.default];
          })
        ];
      };
    };
  };
}
```

</details>

In essence, add `github:x3ro/ahiru-tpm` to your inputs:

```nix
{
  inputs = {
    ahiru-tpm = {
      # Use the GitHub mirror here because using the Codeberg url fails for some reason
      url = "github:x3ro/ahiru-tpm";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  // ...
}
```

Then add the following to your packages list:

```nix
ahiru-tpm.packages.${pkgs.system}.default
```

### Build from source

Setup the latest stable Rust toolchain via [rustup](https://rustup.rs/):

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update
```

Clone the repository:

```sh
git clone https://codeberg.org/x3ro/ahiru-tpm.git
cd ahiru-tpm
```

Then build `ahiru-tpm`:

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
ahiru-tpm install
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
ahiru-tpm update
```

### Uninstalling Plugins

To uninstall plugins, first remove them from your tmux config. Then, to clean
up downloaded plugins, run:

```sh
ahiru-tpm clean
```

### Settings

#### Disable Parallel Mode

Sometimes you might want to disable parallel mode, for example to debug a
problem with a plugin or if parallel loading leads to problems for whatever
reason.

This can be done by setting the following in the tmux config:

```tmux
set -g @tpm-parallel 'false'
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

In this section we compare the perforemance of the original **TPM** written in
bash with the one of **Ahiru-TPM** which uses parallel processing and is
written in rust.

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
hyperfine --warmup 5 --runs 25 --shell nu --prepare 'rm -rf ~/.local/share/tmux/plugins' -n 'ahiru-tpm' 'ahiru-tpm install' --prepare 'ls ~/.config/tmux/plugins/ | where ($it.name | path basename) != tpm | each {rm -rf $in.name}' -n 'tpm' 'TMUX_PLUGIN_MANAGER_PATH=($env.HOME)/.config/tmux/plugins/ ~/.config/tmux/plugins/tpm/bin/install_plugins'
```

![Benchmark installing plugins with `ahiru-tpm` vs. `tpm`. `ahiru-tpm` ran 2.54 times faster than `tpm`.](.res/hyperfine-tpm-install.png)
_Note: The screenshot shows **Ahiru-TPM** as `tpm-rs`, which was the prototype name._

### 5x Faster Plugins Loading

```sh
hyperfine --warmup 10 --runs 100 --shell nu -n 'ahiru-tpm' 'ahiru-tpm load' -n 'tpm' 'TMUX_PLUGIN_MANAGER_PATH=($env.HOME)/.config/tmux/plugins/ ~/.config/tmux/plugins/tpm/scripts/source_plugins.sh'
```

![Benchmark loading plugins with `ahiru-tpm` vs. `tpm`. `ahiru-tpm` ran 5.62 times faster than `tpm`.](.res/hyperfine-tpm-load.png)
_Note: The screenshot shows **Ahiru-TPM** as `tpm-rs`, which was the prototype name._

## License

[GNU GPLv3](./LICENSE)
