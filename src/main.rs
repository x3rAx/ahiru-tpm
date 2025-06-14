use ahiru_tpm::{
    args::{Action, Args},
    plugins::{self},
};
use anyhow::Result;
use clap::Parser;

//#[cmd_lib::main]
fn main() -> Result<()> {
    let args = Args::parse();

    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    match args.action {
        Action::Install(install_arga) => plugins::install::install_cmd(install_arga),
        Action::Update(update_args) => plugins::update::update_cmd(update_args),
        Action::Load => plugins::load::load_cmd(),
        Action::Clean => plugins::clean::clean_cmd(),
        Action::Sync => plugins::sync_cmd(),
    }
}
