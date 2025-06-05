use anyhow::{Context, Result};
use cmd_lib::run_cmd;
use tpm_rs::{
    plugin::Plugin,
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
        let plugin = Plugin::from(spec);
        let url: Url = plugin.url();
        let name = plugin.name();

        println!("\nInstalling {}", plugin);

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
