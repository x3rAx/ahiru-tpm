use anyhow::{Context, Result};
use cmd_lib::run_cmd;
use glob::glob;
use is_executable::IsExecutable;
use log::info;
use rayon::prelude::*;

use crate::{plugin::Plugin, tmux};

pub fn load() -> Result<()> {
    tmux::setup_keymaps()?;
    let plugins = super::get_plugins()?;
    let (parallel, non_parallel) = plugins.into_iter().partition::<Vec<_>, _>(|p| p.parallel());

    non_parallel.iter().try_for_each(load_plugin)?;
    parallel.par_iter().try_for_each(load_plugin)
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
        run_cmd!($init_file)
            .or_else(|_| run_cmd!(/bin/sh $init_file))
            .context(format!(r#"Failed to load plugin "{}""#, plugin.name()))?;
    }
    Ok(())
}
