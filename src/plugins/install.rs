use anyhow::{Context, Error, Result};
use cmd_lib::run_cmd;
use futures::{StreamExt, stream::FuturesUnordered};
use tokio::task;

use crate::{plugin::Plugin, tmux::ensure_plugins_dir_exists};

enum InstallResult {
    Installed,
    Skipped,
}

pub async fn install() -> Result<()> {
    if super::do_parallel() {
        install_parallel().await
    } else {
        super::get_plugins()?.iter().try_for_each(|p| {
            install_plugin(p).context("")?;
            Ok(())
        })
    }
}

async fn install_parallel() -> Result<()> {
    let mut tasks: FuturesUnordered<task::JoinHandle<std::result::Result<(), Error>>> =
        FuturesUnordered::new();
    let plugins = super::get_plugins()?;

    for plugin in plugins {
        tasks.push(task::spawn(async move {
            match install_plugin(&plugin)? {
                InstallResult::Installed => println!("Installed: {plugin}"),
                InstallResult::Skipped => (),
            };
            Ok(())
        }));
    }

    while let Some(result) = tasks.next().await {
        result.context("Task paniced!")?.context("Task failed")?;
    }

    Ok(())
}

fn install_plugin(plugin: &Plugin) -> Result<InstallResult> {
    if plugin.is_installed() {
        return Ok(InstallResult::Skipped);
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
    .context(format!(r#"Failed to install plugin "{}""#, plugin.name()))?;

    Ok(InstallResult::Installed)
}
