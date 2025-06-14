<div align="center">
    <figure>
        <img src=".res/duck_using_tmux_chatgpt.png"
            alt="A duck using tmux (generated with ChatGPT)" width="500">
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

- [Official Mirrors](#official-mirrors)
- [Installation](#installation)
    * [Nix Flakes](#nix-flakes)
    * [Build from source](#build-from-source)
- [Usage](#usage)
    * [Installing Plugins](#installing-plugins)
        + [Plugin Spec](#plugin-spec)
            - [Branch](#branch)
            - [Attributes](#attributes)
    * [Updating Plugins](#updating-plugins)
    * [Uninstalling Plugins](#uninstalling-plugins)
    * [Sync (Install, Clean and Update)](#sync-install-clean-and-update)
    * [Settings](#settings)
        + [Disable Parallel Mode](#disable-parallel-mode)
    * [Key Bindings](#key-bindings)
- [Benchmark](#benchmark)
    * [2x Faster Plugins Installation](#2x-faster-plugins-installation)
    * [5x Faster Plugins Loading](#5x-faster-plugins-loading)
- [License](#license)

<!-- tocstop -->

## Official Mirrors

- **Codeberg: [x3ro/ahiru-tpm](https://codeberg.org/x3ro/ahiru-tpm)**
- GitHub: [x3rAx/ahiru-tpm](https://github.com/x3rAx/ahiru-tpm) (mirror)

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
    # set -g @plugin 'codeberg:user/plugin_name; alias = desired_name'
    # set -g @plugin 'https://codeberg.org/user/plugin_name'
    # set -g @plugin 'git@codeberg.org:user/plugin_name'

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
      url = "github:x3rAx/ahiru-tpm";
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

In essence, add `github:x3rAx/ahiru-tpm` to your inputs:

```nix
{
  inputs = {
    ahiru-tpm = {
      # Use the GitHub mirror here because using the Codeberg url fails for some reason
      url = "github:x3rAx/ahiru-tpm";
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

Plugins can be installed by first adding them to to the tmux config and then
using the [install key-binding](#key-bindings) or by running:

```sh
ahiru-tpm install
```

No need to reload tmux config first. **Ahiru-TPM** parses the tmux config by
itself to detect changes.

To add a plugin, add the following line to your tmux config:

```tmux
set -g @plugin '<plugin-spec>'
```

Where `<plugin-spec>` is described below:

#### Plugin Spec

The general syntax for the plugin specs is:

```text
<git-repo-url>[#branch][; attribute1 = value1[[, attributeN = valueN]...]]
```

Plugins can be installed from everywhere by pasting their full git-repo URL.
There are however some shortcuts possible:

| Example               | Description                                         |
| --------------------- | --------------------------------------------------- |
| `user/repo`           | Short URL [GitHub](https://github.com/) repos       |
| `codeberg:user/repo`  | Short URL [Codeberg](https://codeberg.org/) repos   |
| `github:user/repo`    | Short URL [GitHub](https://github.com/) repos       |
| `gitlab:user/repo`    | Short URL [GitLab](https://gitlab.com/) repos       |
| `bitbucket:user/repo` | Short URL [BitBucket](https://bitbucket.org/) repos |

##### Branch

You can install a plugin from a specific branch by appending it to the repo
URL, separated by a `#`:

> [!Note]
>
> This works for all URLs, not just for short GitHub URLs

```text
user/repo#branch
```

##### Attributes

You can add several attributes to a plugin, that change how it is handled.

Attributes follow after the plugin URL (and branch, if any) from which they are
separated by a `;`. Multiple attributes are sparated by a `,`. Should you want
to use spaces in an attribute value, enclose it in quotes. Here are some
examples:

```tmux
user/repo; attr1 = value1, attr2 = value2
codeberg:user/repo; attr1 = value1
user/repo#branch; attr1=value1, attr2=value2
https:://codeberg.com/user/repo#branch; attr1 = "value with spaces"
```

Below is a list of possible attributes:

| Attribute  | Example              | Description                                                                |
| ---------- | -------------------- | -------------------------------------------------------------------------- |
| `alias`    | `alias = catppuccin` | Choose a different name for the plugin to prevent collisions.<sup>\*</sup> |
| `parallel` | `parallel = false`   | Control whether to load this plugin in parallel.<sup>\*\*</sup>            |

> <sup>\*</sup>
> The plugin name is determined by the repo name, i.e. the part of the repo URL
> after the last `/`. Some repos might have undesired names (e.g.
> `catppuccin/tmux`, which would result in the plugin name `tmux`) or names
> that collide with that of other plugins.
>
> <sup>\*\*</sup>
> This attribute overrides the global `@tpm-parallel` option, so you could
> disable parallel loading for all plugins and enable it only for specific
> ones.

### Updating Plugins

To update plugins run use the [update key-binding](#key-bindings) or run:

```sh
ahiru-tpm update
```

### Uninstalling Plugins

To uninstall plugins, first remove them from your tmux config. Then, to clean
up downloaded plugins, use the [clean key-binding](#key-bindings) or run:

```sh
ahiru-tpm clean
```

### Sync (Install, Clean and Update)

To synchronize with your tmux config, (i.e. install new plugins, update
existing and clean up removed ones) you can use the [sync
key-binding](#key-bindings) or simply run:

```sh
ahiru-tpm sync
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
| `prefix` + `alt + S` | **S**ync plugins               |

To change the key bindings add the following to your tmux config:

```tmux
set -g @tpm-bind-install 'M-I'
set -g @tpm-bind-update 'M-U'
set -g @tpm-bind-clean 'M-C'
set -g @tpm-bind-sync 'M-S'
```

> [!Note]
>
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
