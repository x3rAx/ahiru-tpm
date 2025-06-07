use anyhow::Result;
use clap::Parser;
use tpm_rs::{
    args::{Action, Args, UpdateArgs},
    plugins::{self},
};

#[cmd_lib::main]
fn main() -> Result<()> {
    let args = Args::parse();

    env_logger::init();

    match args.action {
        Action::Install => plugins::install(),
        Action::Update(UpdateArgs { all: true, .. }) => plugins::update_all(),
        Action::Update(UpdateArgs { all: false, names }) => plugins::update(&names),
        Action::Load => plugins::load(),
        Action::Clean => plugins::clean(),
    }
}
