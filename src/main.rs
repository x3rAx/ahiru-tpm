use anyhow::{Result, anyhow};
use tpm_rs::plugins::install_plugins;

fn main() -> Result<()> {
    // Get first argument
    let action = std::env::args().nth(1).unwrap_or("install".into());

    match action.as_ref() {
        "install" => install_plugins(),
        _ => Err(anyhow!("Unknown action: {action}")),
    }
}
