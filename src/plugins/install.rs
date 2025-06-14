use anyhow::{Context, Result};
use cmd_lib::run_cmd;
use futures::{StreamExt, stream::FuturesUnordered};
use tokio::task;

use crate::{plugin::Plugin, tmux::ensure_plugins_dir_exists};

pub async fn install() -> Result<()> {
    if super::do_parallel() {
        install_parallel().await
    } else {
        super::get_plugins()?.iter().try_for_each(install_plugin)
    }
}

async fn install_parallel() -> Result<()> {
    let mut tasks = FuturesUnordered::new();
    let plugins = super::get_plugins()?;

    for plugin in plugins {
        tasks.push(task::spawn(async move { install_plugin(&plugin) }));
    }

    while let Some(result) = tasks.next().await {
        result.context("Task paniced!")?.context("Task failed")?;
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

    println!("Installing plugin {}", plugin);
    run_cmd!(
        GIT_TERMINAL_PROMPT=0 git clone --single-branch --recursive $[branch_args] $url $path;
    )
    .context(format!(r#"Failed to install plugin "{}""#, plugin.name()))
}
