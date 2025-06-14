pub mod install;
pub mod load;
pub mod update;

use std::{collections::HashSet, path::Path};

use anyhow::{Context, Result, anyhow};
use cached::proc_macro::cached;
use cmd_lib::run_cmd;
use glob::glob;
use itertools::Itertools;
use log::warn;

use crate::{
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
    update::update_all()?;
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
