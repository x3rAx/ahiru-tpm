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
    let install_key = get_option("@tpm-bind-install").or_else(|| get_option("@tpm-install"));
    let update_key = get_option("@tpm-bind-update").or_else(|| get_option("@tpm-update"));
    let clean_key = get_option("@tpm-bind-clean").or_else(|| get_option("@tpm-clean"));

    // --- TPM Compatibility (set only when key-option is not set) ---

    if install_key.is_none() {
        run_cmd!(
            tmux bind-key "I" display-popup r"\
            echo '  /!\   This keymap is deprecated. Please use Prefix+<Alt+I> instead   /!\';
            echo ''
            tpm install
            echo $'\n--- Done. Press ESC to close this popup. ---'
        ";
        )
        .context("Failed to setup install keymap for TPM compatibility")?;
    }

    if update_key.is_none() {
        run_cmd!(
            tmux bind-key "U" display-popup r"
            echo '  /!\   This keymap is deprecated. Please use Prefix+<Alt+I> instead   /!\';
            echo ''
            tpm update --all
            echo $'\n--- Done. Press ESC to close this popup. ---'
        ";
        )
        .context("Failed to setup update keymap for TPM compatibility")?;
    }

    if clean_key.is_none() {
        run_cmd!(
            tmux bind-key "M-u" display-popup r"
            echo '  /!\   This keymap is deprecated. Please use Prefix+<Alt+C> instead   /!\';
            echo ''
            tpm clean
            echo $'\n--- Done. Press ESC to close this popup. ---'
        ";
        )
        .context("Failed to setup clean keymap for TPM compatibility")?;
    }

    // --- TPM-RS sensible keymaps ---

    let install_key = install_key.unwrap_or("M-I".into());
    let update_key = update_key.unwrap_or("M-U".into());
    let clean_key = clean_key.unwrap_or("M-C".into());

    run_cmd!(
        tmux bind-key $install_key display-popup r"tpm install --load; echo $'\n--- Done. Press ESC to close this popup. ---'";
        tmux bind-key $update_key display-popup r"tpm update --all --load; echo $'\n--- Done. Press ESC to close this popup. ---'";
        tmux bind-key $clean_key display-popup r"tpm clean; echo $'\n--- Done. Press ESC to close this popup. ---'";
    )
    .context("Failed to setup keymaps")
}

/// Get the CWD in which tmux was initially started
pub fn get_start_path() -> Result<PathBuf> {
    Ok(PathBuf::from(
        run_fun!(tmux display -pt 0 -F "#{pane_start_path}")
            .context("Failed to get tmux start path")?,
    ))
}
