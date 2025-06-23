use std::{env::current_dir, fs, path::PathBuf};

use anyhow::{Context, Result, anyhow};
use cached::proc_macro::cached;
use cmd_lib::run_fun;

use crate::key_bindings;

pub fn get_option(name: &str) -> Option<String> {
    run_fun!(tmux show-option -vg $name 2>/dev/null).ok()
}

pub fn get_existing_config_paths() -> Vec<PathBuf> {
    let mut configs = vec![];

    let system_config = PathBuf::from("/etc/tmux.conf");
    if system_config.exists() {
        configs.push(system_config);
    }

    let home_config = xdir::home()
        .expect("$HOME should be set")
        .join(".tmux.conf");
    if home_config.exists() {
        configs.push(home_config);
    }

    let user_config = xdir::config()
        .expect("XDG paths should be possible to build")
        .join("tmux/tmux.conf");
    if user_config.exists() {
        configs.push(user_config);
    }

    configs
}

pub fn get_plugins_dir() -> PathBuf {
    xdir::data()
        .expect("XDG paths should be possible to build")
        .join("tmux/plugins")
}

pub fn ensure_plugins_dir_exists() -> Result<PathBuf> {
    let path = get_plugins_dir();

    if !path.exists() {
        fs::create_dir_all(&path)?;
    }

    if !path.is_dir() {
        return Err(anyhow!("Not a directory: {}", path.to_string_lossy()));
    }

    Ok(path)
}

pub fn setup_keymaps() -> Result<()> {
    key_bindings::setup()
}

/// Get the CWD in which tmux was initially started
pub fn get_start_path() -> Result<PathBuf> {
    match run_fun!(tmux display -pt 0 -F "#{pane_start_path}") {
        Ok(path) => Ok(PathBuf::from(path)),
        Err(err) => current_dir().with_context(|| {
            format!("Failed to get current directory after getting tmux start path failed: {err}")
        }),
    }
}

#[cached]
pub fn is_tmux_running() -> bool {
    run_fun!(tmux display -p).is_ok()
}
