use clap::{Parser, Subcommand};

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
    Update,
    /// Load plugins defined in tmux config
    Load,
}
