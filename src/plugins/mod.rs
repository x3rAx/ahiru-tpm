pub mod install;
pub mod load;

use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use anyhow::{Context, Result, anyhow};
use cached::proc_macro::cached;
use cmd_lib::run_cmd;
use glob::glob;
use itertools::Itertools;
use log::warn;
use rayon::prelude::*;

use crate::{
    args::UpdateArgs,
    plugin::Plugin,
    spec::Spec,
    tmux::{self},
    tmux_config_parser::{self, ConfigDirective},
    utils,
};

pub fn load_specs() -> Result<Vec<Spec>> {
    let configs = tmux::get_existing_config_paths();

    if configs.is_empty() {
        return Err(anyhow!("Failed to find any tmux config files"));
    };

    let specs: Vec<Spec> = configs
        .into_iter()
        .map(|p| {
            get_specs_from_config(&p).context(format!(
                "Failed to load specs from config file: {}",
                p.display()
            ))
        })
        .flatten_ok()
        .collect::<Result<Vec<_>>>()?;

    Ok(specs)
}

fn get_specs_from_config(path: &Path) -> Result<Vec<Spec>> {
    let it = tmux_config_parser::parse(path)
        .context(format!(
            "Failed to parse specs from config file: {}",
            path.display()
        ))?
        .into_iter()
        .map(|directive| match directive {
            ConfigDirective::PluginSpec(spec) => Ok(vec![spec]),
            ConfigDirective::Source(path) => get_specs_from_config(path.as_ref()),
        })
        .flatten_ok()
        .collect::<Result<Vec<_>>>()?;

    Ok(it)
}

pub fn get_plugins() -> Result<Vec<Plugin>> {
    let legacy_plugins: Vec<Plugin> = tmux::get_option("@tpm_plugins")
        .unwrap_or("".to_owned())
        .split(' ')
        .filter(|&spec| !spec.is_empty())
        .map(Spec::try_from_legacy)
        .map_ok(Plugin::from)
        .collect::<Result<_>>()?;

    let plugins: Vec<Plugin> = load_specs()?.into_iter().map(Plugin::from).collect();

    let plugins: Vec<Plugin> = legacy_plugins.into_iter().chain(plugins).collect();

    utils::ensure_unique_by_key(&plugins, |p| p.name().to_owned()).map_err(|plugin| {
        anyhow!(
            r#"More than one plugin with the name "{}" has been specified"#,
            plugin.name()
        )
    })?;

    Ok(plugins)
}

pub fn update_cmd(args: UpdateArgs) -> Result<()> {
    if args.all {
        update_all()?;
    } else {
        update_list(&args.names)?;
    }

    if args.load {
        load::load_cmd()?;
    }

    println!("==> Done updating plugins.");
    if args.load {
        println!("==> Plugins have been reloaded.");
    }

    Ok(())
}

fn update_all() -> Result<()> {
    let plugins: Vec<Plugin> = get_plugins()?
        .into_iter()
        .filter(|plugin| plugin.is_installed())
        .collect();

    if do_parallel() {
        plugins.par_iter().try_for_each(update_plugin)
    } else {
        plugins.iter().try_for_each(update_plugin)
    }
}

fn update_list<T: AsRef<str>>(names: &[T]) -> Result<()> {
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
        GIT_TERMINAL_PROMPT=0 git pull --rebase;
        GIT_TERMINAL_PROMPT=0 git submodule update --init --recursive;

    )
    .context(format!(r#"Failed to update plugin "{}""#, plugin.name()))
}

pub fn clean_cmd() -> Result<()> {
    clean()?;

    println!();
    println!("==> Done cleaning plugins.");

    Ok(())
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

pub fn sync_cmd() -> Result<()> {
    sync()?;
    load::load()?;

    println!();
    println!("==> Done syncing plugins.");
    println!("==> Plugins have been reloaded.");

    Ok(())
}

pub fn sync() -> Result<()> {
    install::install()?;
    clean_cmd()?;
    update_all()?;
    Ok(())
}

#[cached]
pub fn do_parallel() -> bool {
    let default = true;

    let Some(val) = tmux::get_option("@tpm-parallel") else {
        return default;
    };

    utils::parse_bool(&val).unwrap_or_else(|| {
        warn!(r#"Invalid value "{val}" for option `@tpm-parallel`. Falling back to "true""#);
        default
    })
}
