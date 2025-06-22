use clap::{ArgGroup, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub action: Action,
}

#[derive(Subcommand, Debug)]
pub enum Action {
    /// Install plugins defined in tmux config
    Install(InstallArgs),
    /// Update installed plugins that are defined in tmux config
    Update(UpdateArgs),
    /// Load plugins defined in tmux config
    Load,
    /// Clean plugins not listed in tmux config
    Clean,
    /// Run `install`, `clean` and `update --all` in one go
    Sync,
    /// Initialize Ahiru-TPM and load plugins
    Init,
}

#[derive(Parser, Debug)]
pub struct InstallArgs {
    /// Load plugins after install
    #[arg(short, long)]
    pub load: bool,
}

#[derive(Parser, Debug)]
#[command(group(
    ArgGroup::new("target")
        .args(["all", "names"])
        .required(true)
))]
pub struct UpdateArgs {
    /// Update all
    #[arg(short, long)]
    pub all: bool,

    /// Load plugins after update
    #[arg(short, long)]
    pub load: bool,

    /// List of plugins to update
    pub names: Vec<String>,
}
