use std::{fs, path::PathBuf};

use anyhow::{Context, Result, anyhow};
use cmd_lib::{run_cmd, run_fun};

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
    // TPM Compatibility
    run_cmd!(
        tmux bind-key "I" display-popup r"\
            echo '  /!\   This keymap is deprecated. Please use Prefix+<Alt+I> instead   /!\';
            echo ''
            tpm install
            echo $'\n--- Done. Press ESC to close this popup. ---'
        ";

        tmux bind-key "U" display-popup r"
            echo '  /!\   This keymap is deprecated. Please use Prefix+<Alt+I> instead   /!\';
            echo ''
            tpm update --all
            echo $'\n--- Done. Press ESC to close this popup. ---'
        ";

        tmux bind-key "M-u" display-popup r"
            echo '  /!\   This keymap is deprecated. Please use Prefix+<Alt+C> instead   /!\';
            echo ''
            tpm clean
            echo $'\n--- Done. Press ESC to close this popup. ---'
        ";
    )
    .context("Failed to setup keymaps for TPM compatibility")?;

    // TPM-RS sensible keymaps
    run_cmd!(
        tmux bind-key "M-I" display-popup r"tpm install; echo $'\n--- Done. Press ESC to close this popup. ---'";
        tmux bind-key "M-U" display-popup r"tpm update --all; echo $'\n--- Done. Press ESC to close this popup. ---'";
        tmux bind-key "M-C" display-popup r"tpm clean; echo $'\n--- Done. Press ESC to close this popup. ---'";
    )
    .context("Failed to setup keymaps")
}
