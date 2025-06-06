use clap::Parser;

use crate::action::Action;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub action: Action,
}
