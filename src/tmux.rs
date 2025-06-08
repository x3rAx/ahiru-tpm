use std::path::PathBuf;

use cmd_lib::run_fun;

pub fn get_option(name: &str) -> Option<String> {
    run_fun!(tmux show-option -vg $name 2>/dev/null).ok()
}

// TODO:
//   - Also load from `/etc/tmux.conf`
//   - Get sourced files from tmux and also load from them
pub fn get_config_path() -> Option<PathBuf> {
    get_config_dir().map(|p| p.join("tmux.conf"))
}

pub fn get_plugins_dir() -> Option<PathBuf> {
    get_config_dir().map(|p| p.join("plugins"))
}

pub fn get_config_dir() -> Option<PathBuf> {
    Some(get_user_config_path()?.parent()?.to_owned())
}

fn get_user_config_path() -> Option<PathBuf> {
    // Try `$XDG_CONFIG_HOME` (with fallback to `$HOME/.config/`)
    let config_path = xdir::config().map(|path| path.join("tmux/tmux.conf"));
    if let Some(path) = config_path {
        if path.exists() {
            return Some(path);
        }
    }

    // Try `$HOME/.tmux`
    let config_path = xdir::home().map(|path| path.join(".tmux.conf"));
    if let Some(path) = config_path {
        if path.exists() {
            return Some(path);
        }
    }

    None
}
