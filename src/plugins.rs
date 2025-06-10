use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use anyhow::{Context, Result, anyhow};
use cmd_lib::run_cmd;
use glob::glob;
use is_executable::IsExecutable;
use itertools::Itertools;
use log::{info, warn};
use rayon::prelude::*;

use crate::{
    args::{InstallArgs, UpdateArgs},
    plugin::Plugin,
    spec::Spec,
    tmux::{self, ensure_plugins_dir_exists},
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
            "Failed to load specs from config file: {}",
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
        .map(Spec::try_from_url)
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

pub fn install(args: InstallArgs) -> Result<()> {
    if do_parallel() {
        get_plugins()?.par_iter().try_for_each(install_plugin)
    } else {
        get_plugins()?.iter().try_for_each(install_plugin)
    }?;

    if args.load {
        load()?;
    }

    println!("==> Done installing plugins.");
    if args.load {
        println!("==> Plugins have been reloaded.");
    }

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

pub fn update(args: UpdateArgs) -> Result<()> {
    if args.all {
        update_all()?;
    } else {
        update_list(&args.names)?;
    }

    if args.load {
        load()?;
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
