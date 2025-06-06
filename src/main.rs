use anyhow::Result;
use clap::Parser;
use tpm_rs::{
    args::{Action, Args},
    plugins::{self},
};

fn main() -> Result<()> {
    let args = Args::parse();

    match args.action {
        Action::Install => plugins::install(),
        Action::Update => plugins::update_all(),
        Action::Load => plugins::load(),
    }
}
