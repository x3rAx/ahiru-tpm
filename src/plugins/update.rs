use std::collections::HashMap;

use anyhow::{Context, Result, anyhow};
use cmd_lib::run_cmd;
use rayon::prelude::*;

use crate::{args::UpdateArgs, plugin::Plugin};

pub fn update_cmd(args: UpdateArgs) -> Result<()> {
    if args.all {
        update_all()?;
    } else {
        update_list(&args.names)?;
    }

    if args.load {
        super::load::load_cmd()?;
    }

    println!("==> Done updating plugins.");
    if args.load {
        println!("==> Plugins have been reloaded.");
    }

    Ok(())
}

pub fn update_all() -> Result<()> {
    let plugins: Vec<Plugin> = super::get_plugins()?
        .into_iter()
        .filter(|plugin| plugin.is_installed())
        .collect();

    if super::do_parallel() {
        plugins.par_iter().try_for_each(update_plugin)
    } else {
        plugins.iter().try_for_each(update_plugin)
    }
}

pub fn update_list<T: AsRef<str>>(names: &[T]) -> Result<()> {
    let plugin_map: HashMap<_, _> = super::get_plugins()?
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

    if super::do_parallel() {
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
