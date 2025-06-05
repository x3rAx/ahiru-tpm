use anyhow::Result;
use tpm_rs::{
    plugins::{self, install_plugins},
    tmux::{self, get_option},
};

fn main() -> Result<()> {
    let tpm_async = get_option("@tpm_async").unwrap_or("".into()) == "true";

    println!("@tpm_async: {}", tpm_async);

    let specs = plugins::load_specs()?;

    println!("\nPlugins:");
    for spec in &specs {
        println!("  {}", spec.url());
    }

    println!("\nPlugins dir: {:?}", tmux::get_plugins_dir());

    install_plugins()?;

    println!("\nDone\n");

    Ok(())
}
