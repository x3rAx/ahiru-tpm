use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use anyhow::{Context, Result, anyhow};
use cmd_lib::run_cmd;
use glob::glob;
use is_executable::IsExecutable;
use log::{info, warn};
use pest::Parser;
use pest_derive::Parser;
use rayon::prelude::*;
use regex::Regex;

use crate::{plugin::Plugin, spec::Spec, tmux};

#[derive(Parser)]
#[grammar = "quoted-string.pest"]
struct QuotedParser;

pub fn load_specs() -> Result<Vec<Spec>> {
    let configs = tmux::get_existing_config_paths();

    if configs.is_empty() {
        return Err(anyhow!("Failed to find any tmux config files"));
    };

    tmux::get_existing_config_paths()
        .into_iter()
        .map(|p| load_specs_from_config(&p).context("Failed to load specs from config file"))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .map(|s| Spec::try_from(s?))
        .collect::<Result<Vec<_>>>()
}

fn load_specs_from_config(
    config_path: &Path,
) -> Result<impl Iterator<Item = Result<String>> + use<>> {
    let file = File::open(config_path)?;
    let reader = BufReader::new(file);
    let re = Regex::new(r#"^\s*set(-option)?\s+-g\s+@plugin\s+(?P<spec>.*)"#)?;

    Ok(reader
        .lines()
        .filter_map(filter_and_get_raw_plugin_spec(re))
        .map(|s| parse_quoted_string(s?)))
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
    if do_parallel() {
        get_plugins()?.par_iter().try_for_each(install_plugin)
    } else {
        get_plugins()?.iter().try_for_each(install_plugin)
    }
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
        git clone --single-branch --recursive $[branch_args] $url $path;
    )
    .context(format!(r#"Failed to install plugin "{}""#, plugin.name()))
}

pub fn load() -> Result<()> {
    if do_parallel() {
        get_plugins()?.par_iter().try_for_each(load_plugin)
    } else {
        get_plugins()?.iter().try_for_each(load_plugin)
    }
}

fn load_plugin(plugin: &Plugin) -> Result<()> {
    info!("Loading plugin {}", plugin.name());

    let path_str = plugin.path()?.to_str().context("Path is not valid UTF-8")?;

    // Find all plugin init files (executable files ending in `.tmux`)
    for entry in glob(&format!("{path_str}/*.tmux"))? {
        let entry = entry?;
        if !entry.is_file() || !entry.is_executable() {
            continue;
        }

        let init_file = entry.to_str().context("Path is not valid UTF-8")?;

        // Run the init plugin file
        run_cmd!($init_file).context(format!(r#"Failed to load plugin "{}""#, plugin.name()))?;
    }
    Ok(())
}

pub fn update_all() -> Result<()> {
    let plugins = get_plugins()?
        .into_iter()
        .filter_map(|plugin| match plugin.is_installed() {
            Ok(true) => Some(Ok(plugin)),
            Ok(false) => None,
            Err(e) => Some(Err(e)),
        })
        .collect::<Result<Vec<_>>>()?;

    if do_parallel() {
        plugins.par_iter().try_for_each(update_plugin)
    } else {
        plugins.iter().try_for_each(update_plugin)
    }
}

pub fn update<T: AsRef<str>>(names: &[T]) -> Result<()> {
    let plugin_map: HashMap<_, _> = get_plugins()?
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

    if do_parallel() {
        plugins
            .par_iter()
            .try_for_each(|plugin| update_plugin(plugin))
    } else {
        plugins.iter().try_for_each(|plugin| update_plugin(plugin))
    }
}

fn update_plugin(plugin: &Plugin) -> Result<()> {
    if !plugin.is_installed()? {
        return Err(anyhow!(r#"Plugin "{}" is not installed"#, plugin.name()));
    }

    let path = plugin.path()?;

    run_cmd!(
        cd $path;
        git pull;
        git submodule update --init --recursive;

    )
    .context(format!(r#"Failed to update plugin "{}""#, plugin.name()))
}

pub fn clean() -> Result<()> {
    let plugin_set: HashSet<_> = get_plugins()?
        .into_iter()
        .map(|plugin| {
            Ok(plugin
                .path()?
                .to_str()
                .context("Path is not valid UTF-8")?
                .to_owned())
        })
        .collect::<Result<HashSet<_>>>()?;

    let plugin_dir = tmux::get_plugins_dir().context("Failed to find tmux plugins dir")?;
    let plugin_dir_str = plugin_dir.to_str().context("Path is not valid UTF-8")?;

    for entry in glob(&format!("{plugin_dir_str}/*"))? {
        let entry = entry?;
        let path_str = entry.to_str().context("Path is not valid UTF-8")?;

        if !plugin_set.contains(path_str) {
            println!("Removing {path_str}");
            run_cmd!(rm -rf $path_str)?
        }
    }
    Ok(())
}

fn do_parallel() -> bool {
    match tmux::get_option("@tpm_parallel").as_deref() {
        None => true,
        Some("true") | Some("yes") => true,
        Some("false") | Some("no") => false,
        Some(val) => {
            warn!(r#"Invalid value "{val}" for option `@tpm_parallel`. Falling back to "true""#);
            true
        }
    }
}
