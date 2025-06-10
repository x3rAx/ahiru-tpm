use anyhow::{Context, Result};
use cmd_lib::run_cmd;

use crate::tmux::get_option;

struct KeyBindings {
    install_key: Option<String>,
    update_key: Option<String>,
    clean_key: Option<String>,
}

impl KeyBindings {
    fn from_tmux() -> Self {
        Self {
            install_key: get_option("@tpm-bind-install").or_else(|| get_option("@tpm-install")),
            update_key: get_option("@tpm-bind-update").or_else(|| get_option("@tpm-update")),
            clean_key: get_option("@tpm-bind-clean").or_else(|| get_option("@tpm-clean")),
        }
    }
}

pub fn setup() -> Result<()> {
    let binds = KeyBindings::from_tmux();

    // --- TPM Compatibility (set only when key-option is not set) ---

    setup_tpm_compatibility(&binds)?;

    // --- Ahiru-TPM sensible keymaps ---

    let install_key = binds.install_key.unwrap_or("M-I".into());
    let update_key = binds.update_key.unwrap_or("M-U".into());
    let clean_key = binds.clean_key.unwrap_or("M-C".into());

    run_cmd!(
        tmux bind-key $install_key display-popup r"ahiru-tpm install --load; echo $'\n--- Done. Press ESC to close this popup. ---'";
        tmux bind-key $update_key display-popup r"ahiru-tpm update --all --load; echo $'\n--- Done. Press ESC to close this popup. ---'";
        tmux bind-key $clean_key display-popup r"ahiru-tpm clean; echo $'\n--- Done. Press ESC to close this popup. ---'";
    )
    .context("Failed to setup keymaps")
}

fn setup_tpm_compatibility(key_maps: &KeyBindings) -> Result<()> {
    if key_maps.install_key.is_none() {
        run_cmd!(
            tmux bind-key "I" display-popup r"\
            echo '  /!\   This keymap is deprecated. Please use Prefix+<Alt+I> instead   /!\';
            echo ''
            ahiru-tpm install
            echo $'\n--- Done. Press ESC to close this popup. ---'
        ";
        )
        .context("Failed to setup keymaps for TPM compatibility")?;
    }

    if key_maps.update_key.is_none() {
        run_cmd!(
            tmux bind-key "U" display-popup r"
            echo '  /!\   This keymap is deprecated. Please use Prefix+<Alt+I> instead   /!\';
            echo ''
            ahiru-tpm update --all
            echo $'\n--- Done. Press ESC to close this popup. ---'
        ";
        )
        .context("Failed to setup keymaps for TPM compatibility")?;
    }

    if key_maps.clean_key.is_none() {
        run_cmd!(
            tmux bind-key "M-u" display-popup r"
            echo '  /!\   This keymap is deprecated. Please use Prefix+<Alt+C> instead   /!\';
            echo ''
            ahiru-tpm clean
            echo $'\n--- Done. Press ESC to close this popup. ---'
        ";
        )
        .context("Failed to setup keymaps for TPM compatibility")?;
    }

    Ok(())
}
