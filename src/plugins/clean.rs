use std::collections::HashSet;

use anyhow::{Context, Result};
use cmd_lib::run_cmd;
use glob::glob;

use crate::tmux::{self};

pub fn clean_cmd() -> Result<()> {
    clean()?;

    println!();
    println!("==> Done cleaning plugins.");

    Ok(())
}
pub fn clean() -> Result<()> {
    let plugin_set: HashSet<_> = super::get_plugins()?
        .into_iter()
        .map(|plugin| {
            Ok(plugin
                .path()
                .to_str()
                .context("Path is not valid UTF-8")?
                .to_owned())
        })
        .collect::<Result<HashSet<_>>>()?;

    let plugin_dir = tmux::get_plugins_dir();
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
