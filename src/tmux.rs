use std::path::PathBuf;

use cmd_lib::run_fun;

pub fn get_option(name: &str) -> Option<String> {
    run_fun!(tmux show-option -vg $name 2>/dev/null).ok()
}

pub fn get_existing_config_paths() -> Vec<PathBuf> {
    let mut configs = vec![];

    let system_config = PathBuf::from("/etc/tmux.conf");
    if system_config.exists() {
        configs.push(system_config);
    }

    let user_config = get_user_config_path();
    if user_config.exists() {
        configs.push(user_config);
    }

    let home_config = xdir::home()
        .expect("$HOME should be set")
        .join(".tmux.conf");
    if home_config.exists() {
        configs.push(home_config);
    }

    configs
}

pub fn get_plugins_dir() -> PathBuf {
    xdir::data()
        .expect("XDG paths should be possible to build")
        .join("tmux/plugins")
}


pub fn get_config_dir() -> PathBuf {
    get_user_config_path()
        .parent()
        .expect("User config is a file and should therefore have a parent")
        .to_owned()
}

fn get_user_config_path() -> PathBuf {
    // Try `$XDG_CONFIG_HOME` (with fallback to `$HOME/.config/`)
    let config_path_xdg = xdir::config()
        .expect("XDG paths should be possible to build")
        .join("tmux/tmux.conf");
    if config_path_xdg.exists() {
        return config_path_xdg;
    }

    // Try `$HOME/.tmux`
    let config_path = xdir::home().expect("XDG paths should be possible to build");
    if config_path.exists() {
        return config_path;
    }

    // Use `$XDG_CONFIG_HOME` as fallback
    config_path_xdg
}
