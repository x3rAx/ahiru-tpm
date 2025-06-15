use std::io;

use anyhow::{Context, Error, Result, anyhow};
use cmd_lib::{FunChildren, run_cmd, spawn_with_output};
use futures::{StreamExt, stream::FuturesUnordered};
use tokio::task;

use crate::{plugin::Plugin, prefix_lines::PrefixLines, tmux::ensure_plugins_dir_exists};

struct InstallResult {
    plugin: Plugin,
    result: Result<(), io::Error>,
    stdout: String,
    stderr: String,
}

impl InstallResult {
    fn new(plugin: Plugin, output: (Result<(), io::Error>, String, String)) -> InstallResult {
        InstallResult {
            plugin,
            result: output.0,
            stdout: output.1,
            stderr: output.2,
        }
    }
}

pub async fn install() -> Result<()> {
    let plugins = super::get_plugins()?
        .into_iter()
        .filter(|p| !p.is_installed())
        .collect::<Vec<_>>();

    let results = if super::do_parallel() {
        install_parallel(plugins).await?
    } else {
        install_sequential(plugins)?
    };

    for result in results {
        if let Err(err) = result.result {
            println!();
            println!("Error installing plugin {}", result.plugin);
            println!("--- stdout: ---\n{}\n", result.stdout.prefix_lines("out> "));
            println!("--- stderr: ---\n{}\n", result.stderr.prefix_lines("err> "));
            println!("Error: {}", err);
        }
    }
    Ok(())
}

fn install_sequential(plugins: Vec<Plugin>) -> Result<Vec<InstallResult>> {
    let mut results = vec![];
    for plugin in plugins {
        let result = install_plugin(plugin)?;
        results.push(result);
    }
    Ok(results)
}

async fn install_parallel(plugins: Vec<Plugin>) -> Result<Vec<InstallResult>> {
    let mut tasks: FuturesUnordered<task::JoinHandle<std::result::Result<InstallResult, Error>>> =
        FuturesUnordered::new();

    for plugin in plugins {
        tasks.push(task::spawn(async move {
            println!("Installing plugin {}", plugin);
            install_plugin(plugin)
        }));
    }

    let mut results = vec![];
    while let Some(result) = tasks.next().await {
        let result = result.context("Task panicked!")?.context("Task failed")?;

        if result.result.is_ok() {
            println!("Installed plugin: {}", result.plugin);
        } else {
            println!("Failed to install plugin: {}", &result.plugin);
        }

        results.push(result);
    }

    Ok(results)
}

fn install_plugin(plugin: Plugin) -> Result<InstallResult> {
    if plugin.is_installed() {
        return Err(anyhow!("Plugin already installed: {plugin}"));
    }

    ensure_plugins_dir_exists()?;

    let url = plugin.url();
    let path = plugin.path();

    let branch_args = match plugin.branch() {
        Some(branch) => vec!["--branch", branch],
        None => vec![],
    };

    let mut proc: FunChildren = spawn_with_output!(
        GIT_TERMINAL_PROMPT=0 git clone --single-branch --recursive $[branch_args] $url $path;
    )
    .context(format!(
        r#"Failed to spawn subprocess for installing "{}". Is `git` installed?"#,
        plugin
    ))?;
    let out = proc.wait_with_all();

    Ok(InstallResult::new(plugin, out))
}
