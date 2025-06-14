use ahiru_tpm::{
    args::{Action, Args},
    cmd,
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
        Action::Install(install_arga) => cmd::install(install_arga),
        Action::Update(update_args) => cmd::update(update_args),
        Action::Load => cmd::load(),
        Action::Clean => cmd::clean(),
        Action::Sync => cmd::sync(),
    }
}
