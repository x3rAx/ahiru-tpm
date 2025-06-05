use anyhow::{Context, Result};
use cmd_lib::run_cmd;
use tpm_rs::{
    plugins,
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

    let _plugins_dir = tmux::get_plugins_dir().context("Failed to get tmux plugins dir")?;

    for spec in specs {
        let url = spec.url();
        let name = spec.name();
        println!("\nInstalling {}", spec.name());

        run_cmd!(
            cd /tmp;
            rm -rf tmux-plugins;
            mkdir -p tmux-plugins;
            git clone $url tmux-plugins/$name;
        )?;
    }

    println!("\nDone\n");

    Ok(())
}
