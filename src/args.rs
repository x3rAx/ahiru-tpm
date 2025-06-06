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
    Install,
    /// Update installed plugins that are defined in tmux config
    Update(UpdateArgs),
    /// Load plugins defined in tmux config
    Load,
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

    /// List of plugins to update
    pub names: Vec<String>,
}
