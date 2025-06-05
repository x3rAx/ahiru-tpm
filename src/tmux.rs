use cmd_lib::run_fun;

pub fn get_option(name: &str) -> Option<String> {
    run_fun!(tmux show-option -vg $name).ok()
}

// TODO:
//   - Also load from `/etc/tmux.conf`
//   - Get sourced files from tmux and also load from them
pub fn get_config_path() -> Option<std::path::PathBuf> {
    // Try `$XDG_CONFIG_HOME` (with fallback to `$HOME/.config/`)
    if let Some(config_path) = xdir::config().map(|path| path.join("tmux/tmux.conf")) {
        if config_path.exists() {
            return Some(config_path);
        }
    }

    // Try `$HOME/.tmux`
    if let Some(config_path) = xdir::home().map(|path| path.join("tmux/tmux.conf")) {
        if config_path.exists() {
            return Some(config_path);
        }
    }

    None
}
