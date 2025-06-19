use std::{io, time::Duration};

use anyhow::{Context, Error, Result, anyhow};
use cmd_lib::{FunChildren, spawn_with_output};
use colored::Colorize;
use futures::{StreamExt, stream::FuturesUnordered};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use tokio::task;

use crate::{
    plugin::Plugin, prefix_lines::PrefixLines, tmux::ensure_plugins_dir_exists,
    truncate_ellipsis::TruncateEllipsis,
};

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
        if result.result.is_err() {
            print_error(result);
        }
    }
    Ok(())
}

fn print_error(result: InstallResult) {
    eprintln!();
    eprintln!();

    eprintln!(
        "{}",
        format!(r#"Failed to install plugin "{}""#, result.plugin)
            .bold()
            .red()
    );

    eprint!(
        "{}",
        result
            .stdout
            .prefix_lines(&"  out> ".bold().green().to_string())
    );
    if !result.stdout.is_empty() && !result.stdout.ends_with("\n") {
        eprintln!()
    }

    eprint!(
        "{}",
        result
            .stderr
            .prefix_lines(&"  err-out> ".bold().red().to_string())
    );
    if !result.stdout.is_empty() && !result.stderr.ends_with("\n") {
        eprintln!()
    }
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
    let mp = MultiProgress::new();

    for plugin in plugins {
        let plugin_name = plugin.to_string().truncate_ellipsis(62);

        let pb = mp.add(ProgressBar::new_spinner());
        pb.set_message(
            format!("{:<64} {}", plugin_name, "Installing")
        );
        pb.enable_steady_tick(Duration::from_millis(100));
        pb.set_style(
            ProgressStyle::with_template("{spinner:.cyan} {msg}")
                .context("Failed to set progress style")?,
        );

        tasks.push(task::spawn(async move {
            let res = install_plugin(plugin)?;

            if res.result.is_ok() {
                pb.set_style(
                    ProgressStyle::with_template(&format!("{} {}", "✔".bold().green(), "{msg}"))
                        .context("Failed to set progress style")?,
                );
                pb.finish_with_message(
                    format!("{:<64} {}", plugin_name, "Done"),
                );
            } else {
                pb.set_style(
                    ProgressStyle::with_template(&format!("{} {}", "✘".bold().red(), "{msg}"))
                        .context("Failed to set progress style")?,
                );
                pb.finish_with_message(
                    format!("{:<64} {}", plugin_name, "Failed".bold().red()),
                );
            }

            Ok(res)
        }));
    }

    let mut results = vec![];
    while let Some(result) = tasks.next().await {
        let result = result.context("Task panicked!")?.context("Task failed")?;

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
