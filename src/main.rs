use anyhow::Result;
use tpm_rs::plugins::install_plugins;

fn main() -> Result<()> {
    install_plugins()?;
    Ok(())
}
