use std::path::PathBuf;

use cmd_lib::run_fun;

pub fn get_option(name: &str) -> Option<String> {
    run_fun!(tmux show-option -vg $name).ok()
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
    // Try `$XDG_CONFIG_HOME` (with fallback to `$HOME/.config/`)
    let config_dir = xdir::config().map(|path| path.join("tmux"));
    if has_tmux_config(&config_dir) {
        return config_dir;
    }

    // Try `$HOME/.tmux`
    let config_dir = xdir::home().map(|path| path.join(".tmux"));
    if has_tmux_config(&config_dir) {
        return config_dir;
    }

    None
}

fn has_tmux_config(config_dir: &Option<PathBuf>) -> bool {
    if let Some(config_path) = config_dir.as_ref().map(|p| p.join("tmux.conf")) {
        return config_path.exists();
    }
    false
}
