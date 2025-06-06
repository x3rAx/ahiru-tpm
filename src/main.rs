use anyhow::Result;
use clap::Parser;
use tpm_rs::{
    action::Action,
    args::Args,
    plugins::{self},
};

fn main() -> Result<()> {
    let args = Args::parse();

    match args.action {
        Action::Install => plugins::install(),
        Action::Load => plugins::load(),
    }
}
