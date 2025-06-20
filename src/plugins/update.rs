use std::collections::HashMap;

use anyhow::{Context, Error, Result, anyhow};
use cmd_lib::{FunChildren, spawn_with_output};
use colored::Colorize;
use futures::{StreamExt, stream::FuturesUnordered};
use tokio::task;

use crate::{
    plugin::Plugin, prefix_lines::PrefixLines, progress_status::ProgressStatus,
    truncate_ellipsis::TruncateEllipsis,
};

use super::action_result::ActionResult;

pub async fn update_all() -> Result<()> {
    let plugins: Vec<Plugin> = super::get_plugins()?
        .into_iter()
        .filter(|plugin| plugin.is_installed())
        .collect();

    update_plugins(plugins).await
}

pub async fn update_list<T: AsRef<str>>(names: &[T]) -> Result<()> {
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

    update_plugins(plugins).await
}

async fn update_plugins(plugins: Vec<Plugin>) -> Result<()> {
    let results = if super::do_parallel() {
        update_parallel(plugins).await?
    } else {
        update_sequential(plugins)?
    };

    for result in results {
        if result.result.is_err() {
            print_error(result);
        }
    }
    Ok(())
}

fn update_sequential(plugins: Vec<Plugin>) -> Result<Vec<ActionResult>> {
    let mut results = vec![];

    let progress = ProgressStatus::new();
    let mut plugin_progresses = HashMap::new();

    for plugin in &plugins {
        let plugin_name = plugin.to_string().truncate_ellipsis(62);
        let pt = progress.add_task(&plugin_name, "Waiting")?;
        plugin_progresses.insert(plugin.to_string(), pt);
    }

    for plugin in plugins {
        let pt = plugin_progresses
            .get(&plugin.to_string())
            .expect("Plugin progress should be in map");

        pt.set_status("Updating")?;

        let res = update_plugin(plugin)?;

        if res.result.is_ok() {
            pt.set_success("Done")?;
        } else {
            pt.set_failed("Failed")?;
        }

        results.push(res);
    }

    Ok(results)
}

async fn update_parallel(plugins: Vec<Plugin>) -> Result<Vec<ActionResult>> {
    let mut tasks: FuturesUnordered<task::JoinHandle<std::result::Result<ActionResult, Error>>> =
        FuturesUnordered::new();
    let progress = ProgressStatus::new();

    for plugin in plugins {
        let plugin_name = plugin.to_string().truncate_ellipsis(62);
        let pt = progress.add_task(&plugin_name, "Waiting")?;

        tasks.push(task::spawn(async move {
            pt.set_status("Updating")?;

            let res = update_plugin(plugin)?;

            if res.result.is_ok() {
                pt.set_success("Done")?;
            } else {
                pt.set_failed("Failed")?;
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

fn update_plugin(plugin: Plugin) -> Result<ActionResult> {
    if !plugin.is_installed() {
        return Err(anyhow!(r#"Plugin "{}" is not installed"#, plugin.name()));
    }

    let path = plugin.path();

    let mut proc: FunChildren = spawn_with_output!(
        bash -c "
            cd $path;
            GIT_TERMINAL_PROMPT=0 git pull --rebase;
            GIT_TERMINAL_PROMPT=0 git submodule update --init --recursive;
        "
    )
    .context(format!(
        r#"Failed to spawn subprocess for updating "{}". Is `git` installed?"#,
        plugin
    ))?;
    let out = proc.wait_with_all();

    Ok(ActionResult::new(plugin, out))
}

fn print_error(result: ActionResult) {
    eprintln!();
    eprintln!();

    eprintln!(
        "{}",
        format!(r#"Failed to update plugin "{}""#, result.plugin)
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
