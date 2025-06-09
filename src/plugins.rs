use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result, anyhow};
use cmd_lib::run_cmd;
use glob::glob;
use is_executable::IsExecutable;
use itertools::Itertools;
use log::{info, warn};
use rayon::prelude::*;

use crate::{
    plugin::Plugin,
    spec::Spec,
    tmux::{self, ensure_plugins_dir_exists},
    tmux_config_parser::{self, ConfigDirective},
};

pub fn load_specs() -> Result<Vec<Spec>> {
    let configs = tmux::get_existing_config_paths();

    if configs.is_empty() {
        return Err(anyhow!("Failed to find any tmux config files"));
    };

    configs
        .into_iter()
        .map(|p| {
            println!("1");
            tmux_config_parser::parse(&p).context(format!(
                "Failed to load specs from config file: {}",
                p.display()
            ))
        })
        .flatten_ok()
        .map_ok(|d| match d {
            ConfigDirective::PluginSpec(spec) => spec,
        })
        .collect::<Result<_>>()
}

pub fn get_plugins() -> Result<Vec<Plugin>> {
    Ok(load_specs()?.into_iter().map(Plugin::from).collect())
}

pub fn install() -> Result<()> {
    if do_parallel() {
        get_plugins()?.par_iter().try_for_each(install_plugin)
    } else {
        get_plugins()?.iter().try_for_each(install_plugin)
    }?;

    load()?;

    println!("==> Done installing plugins.");
    println!("==> Plugins have been reloaded.");
    Ok(())
}

fn install_plugin(plugin: &Plugin) -> Result<()> {
    if plugin.is_installed() {
        return Ok(());
    }

    ensure_plugins_dir_exists()?;

    let url = plugin.url();
    let path = plugin.path();

    let branch_args = match plugin.branch() {
        Some(branch) => vec!["--branch", branch],
        None => vec![],
    };

    run_cmd!(
        git clone --single-branch --recursive $[branch_args] $url $path;
    )
    .context(format!(r#"Failed to install plugin "{}""#, plugin.name()))
}

pub fn load() -> Result<()> {
    tmux::setup_keymaps()?;

    if do_parallel() {
        get_plugins()?.par_iter().try_for_each(load_plugin)
    } else {
        get_plugins()?.iter().try_for_each(load_plugin)
    }
}

fn load_plugin(plugin: &Plugin) -> Result<()> {
    info!("Loading plugin {}", plugin.name());

    let path_str = plugin.path().to_str().context("Path is not valid UTF-8")?;

    // Find all plugin init files (executable files ending in `.tmux`)
    for entry in glob(&format!("{path_str}/*.tmux"))? {
        let entry = entry?;
        if !entry.is_file() || !entry.is_executable() {
            continue;
        }

        let init_file = entry.to_str().context("Path is not valid UTF-8")?;

        // Run the init plugin file
        run_cmd!($init_file).context(format!(r#"Failed to load plugin "{}""#, plugin.name()))?;
    }
    Ok(())
}

pub fn update_all() -> Result<()> {
    let plugins: Vec<Plugin> = get_plugins()?
        .into_iter()
        .filter(|plugin| plugin.is_installed())
        .collect();

    if do_parallel() {
        plugins.par_iter().try_for_each(update_plugin)
    } else {
        plugins.iter().try_for_each(update_plugin)
    }?;

    load()?;

    println!("==> Done updating plugins.");
    println!("==> Plugins have been reloaded.");
    Ok(())
}

pub fn update<T: AsRef<str>>(names: &[T]) -> Result<()> {
    let plugin_map: HashMap<_, _> = get_plugins()?
        .into_iter()
        .map(|plugin| (plugin.name().to_owned(), plugin))
        .collect();

    let plugins = names
        .iter()
        .map(|name| {
            let name = name.as_ref();
            plugin_map
                .get(name)
                .context(format!("Unknown plugin name: {}", name))
        })
        .collect::<Result<Vec<_>>>()?;

    if do_parallel() {
        plugins
            .par_iter()
            .try_for_each(|plugin| update_plugin(plugin))
    } else {
        plugins.iter().try_for_each(|plugin| update_plugin(plugin))
    }
}

fn update_plugin(plugin: &Plugin) -> Result<()> {
    if !plugin.is_installed() {
        return Err(anyhow!(r#"Plugin "{}" is not installed"#, plugin.name()));
    }

    let path = plugin.path();

    run_cmd!(
        cd $path;
        git pull --rebase;
        git submodule update --init --recursive;

    )
    .context(format!(r#"Failed to update plugin "{}""#, plugin.name()))
}

pub fn clean() -> Result<()> {
    let plugin_set: HashSet<_> = get_plugins()?
        .into_iter()
        .map(|plugin| {
            Ok(plugin
                .path()
                .to_str()
                .context("Path is not valid UTF-8")?
                .to_owned())
        })
        .collect::<Result<HashSet<_>>>()?;

    let plugin_dir = tmux::get_plugins_dir();
    let plugin_dir_str = plugin_dir.to_str().context("Path is not valid UTF-8")?;

    for entry in glob(&format!("{plugin_dir_str}/*"))? {
        let entry = entry?;
        let path_str = entry.to_str().context("Path is not valid UTF-8")?;

        if !plugin_set.contains(path_str) {
            println!("Removing {path_str}");
            run_cmd!(rm -rf $path_str)?
        }
    }
    Ok(())
}

fn do_parallel() -> bool {
    match tmux::get_option("@tpm_parallel").as_deref() {
        None => true,
        Some("true") | Some("yes") => true,
        Some("false") | Some("no") => false,
        Some(val) => {
            warn!(r#"Invalid value "{val}" for option `@tpm_parallel`. Falling back to "true""#);
            true
        }
    }
}
