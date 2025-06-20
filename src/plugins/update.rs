use std::collections::HashMap;

use anyhow::{Context, Result, anyhow};
use cmd_lib::run_cmd;
use rayon::prelude::*;

use crate::plugin::Plugin;

pub fn update_all() -> Result<()> {
    let plugins: Vec<Plugin> = super::get_plugins()?
        .into_iter()
        .filter(|plugin| plugin.is_installed())
        .collect();

    update_plugins(plugins)
}

pub fn update_list<T: AsRef<str>>(names: &[T]) -> Result<()> {
    let mut plugin_map: HashMap<_, _> = super::get_plugins()?
        .into_iter()
        .map(|plugin| (plugin.name().to_owned(), plugin))
        .collect();

    let plugins = names
        .iter()
        .map(|name| {
            let name = name.as_ref();
            plugin_map
                .remove(name)
                .context(format!("Unknown plugin name: {}", name))
        })
        .collect::<Result<Vec<_>>>()?;

    update_plugins(plugins)
}

fn update_plugins(plugins: Vec<Plugin>) -> Result<()> {
    if super::do_parallel() {
        plugins.par_iter().try_for_each(update_plugin)
    } else {
        plugins.iter().try_for_each(update_plugin)
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
