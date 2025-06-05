use anyhow::{Context, Result};
use cmd_lib::run_cmd;
use tpm_rs::{
    plugins,
    tmux::{self, get_option},
};
use url::Url;

fn main() -> Result<()> {
    let tpm_async = get_option("@tpm_async").unwrap_or("".into()) == "true";

    println!("@tpm_async: {}", tpm_async);

    let specs = plugins::load_specs()?;

    println!("\nPlugins:");
    for spec in &specs {
        println!("  {}", spec.url());
    }

    println!("\nPlugins dir: {:?}", tmux::get_plugins_dir());

    let plugins_dir = tmux::get_plugins_dir().context("Failed to get tmux plugins dir")?;

    for spec in specs {
        let url: Url = spec.url().into();
        let name = spec.name();
        println!("\nInstalling {}", spec.name());

        run_cmd!(
            cd /tmp;
            rm -rf tmux-plugins;
            mkdir -p tmux-plugins;
            echo git clone $url $plugins_dir/$name;
        )?;
    }

    println!("\nDone\n");

    Ok(())
}
