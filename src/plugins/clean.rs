use std::{collections::HashSet, fs::remove_dir_all};

use anyhow::{Context, Result};

use crate::tmux::{self};

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

    // List entries in plugin_dir
    for entry in plugin_dir
        .read_dir()
        .context("Failed to read plugin directory")?
    {
        let entry = entry?;
        let file_type = entry.file_type()?;

        if !file_type.is_dir() {
            continue;
        }

        let path = entry.path();
        let path_str = path.to_str().context("Path is not valid UTF-8")?;

        if !plugin_set.contains(path_str) {
            eprintln!("-> Removing {path_str}");
            remove_dir_all(path)?;
        }
    }

    Ok(())
}
