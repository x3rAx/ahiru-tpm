use anyhow::{Context, Result};
use cmd_lib::run_cmd;
use rayon::prelude::*;

use crate::{plugin::Plugin, tmux::ensure_plugins_dir_exists};

pub fn install() -> Result<()> {
    if super::do_parallel() {
        super::get_plugins()?
            .par_iter()
            .try_for_each(install_plugin)
    } else {
        super::get_plugins()?.iter().try_for_each(install_plugin)
    }
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

    println!("Installing plugin {}", plugin);
    run_cmd!(
        GIT_TERMINAL_PROMPT=0 git clone --single-branch --recursive $[branch_args] $url $path;
    )
    .context(format!(r#"Failed to install plugin "{}""#, plugin.name()))
}
