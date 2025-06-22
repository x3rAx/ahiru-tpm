use ahiru_tpm::{
    args::{Action, Args},
    cmd,
};
use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    env_logger::builder()
        .filter_level(log::LevelFilter::Warn)
        .init();

    match args.action {
        Action::Install(install_arga) => cmd::install(install_arga).await,
        Action::Update(update_args) => cmd::update(update_args).await,
        Action::Load => cmd::load(),
        Action::Clean => cmd::clean(),
        Action::Sync => cmd::sync().await,
        Action::Init => cmd::init(),
    }
}
