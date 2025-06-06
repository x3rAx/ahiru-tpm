use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use anyhow::{Context, Result, anyhow};
use cmd_lib::run_cmd;
use glob::glob;
use is_executable::IsExecutable;
use pest::Parser;
use pest_derive::Parser;
use rayon::prelude::*;
use regex::Regex;

use crate::{plugin::Plugin, spec::Spec, tmux};

#[derive(Parser)]
#[grammar = "quoted-string.pest"]
struct QuotedParser;

pub fn load_specs() -> Result<Vec<Spec>> {
    let config_path =
        tmux::get_config_path().ok_or_else(|| anyhow!("Failed to find tmux config file"))?;

    load_specs_from_config(&config_path)?
        .into_iter()
        .map(Spec::try_from)
        .collect::<Result<_, _>>()
}

fn load_specs_from_config(config_path: &Path) -> Result<Vec<String>> {
    let file = File::open(config_path)?;
    let reader = BufReader::new(file);
    let re = Regex::new(r#"^\s*set(-option)?\s+-g\s+@plugin\s+(?P<spec>.*)"#)?;

    reader
        .lines()
        .filter_map(filter_and_get_raw_plugin_spec(re))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(parse_quoted_string)
        .collect::<Result<_, _>>()
}

fn filter_and_get_raw_plugin_spec(
    re: Regex,
) -> impl FnMut(
    std::result::Result<String, std::io::Error>,
) -> Option<std::result::Result<String, std::io::Error>> {
    move |line| match line {
        Ok(l) => re
            .captures(&l)?
            .name("spec")
            .map(|m| Ok(m.as_str().to_string())),
        Err(e) => Some(Err(e)),
    }
}

fn parse_quoted_string(s: String) -> std::result::Result<String, anyhow::Error> {
    match QuotedParser::parse(Rule::quoted_string, &s) {
        Ok(mut pairs) => Ok(pairs
            .next()
            .expect("The single or double inner should always exist")
            .as_str()
            .to_string()),
        Err(e) => Err(e).context("Failed to parse plugins from tmux config"),
    }
}

pub fn get_plugins() -> Result<Vec<Plugin>> {
    Ok(load_specs()?.into_iter().map(Plugin::from).collect())
}

pub fn install() -> Result<()> {
    get_plugins()?
        .par_iter()
        .map(install_plugin)
        .collect::<Result<_>>()
}

fn install_plugin(plugin: &Plugin) -> std::result::Result<(), anyhow::Error> {
    if plugin.is_installed()? {
        return Ok(());
    }

    let url = plugin.url();
    let path = plugin.path()?;

    let branch_args = match plugin.branch() {
        Some(branch) => vec!["--branch", branch],
        None => vec![],
    };

    run_cmd!(
        cd /tmp;
        git clone --single-branch --recursive $[branch_args] $url $path;
    )
    .context(format!("Failed to install plugin {}", plugin.name()))
}

pub fn load() -> Result<()> {
    get_plugins()?
        .par_iter()
        .map(load_plugin)
        .collect::<Result<_>>()
}

fn load_plugin(plugin: &Plugin) -> Result<()> {
    let path_str = plugin.path()?.to_str().context("Path is not valid UTF-8")?;

    // Find all plugin init files (executable files ending in `.tmux`)
    for entry in glob(&format!("{path_str}/*.tmux"))? {
        let entry = entry?;
        if !entry.is_file() || !entry.is_executable() {
            continue;
        }

        let init_file = entry.to_str().context("Path is not valid UTF-8")?;

        // Run the init plugin file
        run_cmd!($init_file).context("Failed to load plugin")?;
    }
    Ok(())
}
