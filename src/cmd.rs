use std::process::exit;

use anyhow::Result;

use crate::{
    args::{InstallArgs, UpdateArgs},
    plugins::{clean, init, install, load, sync, update},
    tmux::is_tmux_running,
};

pub async fn install(args: InstallArgs) -> Result<()> {
    install::install().await?;

    if args.load {
        load()?;
    }

    Ok(())
}

pub async fn update(args: UpdateArgs) -> Result<()> {
    if args.all {
        update::update_all().await?;
    } else {
        update::update_list(&args.names).await?;
    }

    if args.load {
        load()?;
    }

    Ok(())
}

pub fn load() -> Result<()> {
    if !is_tmux_running() {
        eprintln!("WARN: Tmux is not running, plugins are not reloaded.");
        return Ok(());
    }
    load::load()?;

    eprintln!("==> Plugins have been reloaded.");
    Ok(())
}

pub fn clean() -> Result<()> {
    clean::clean()?;

    eprintln!("==> Done");

    Ok(())
}

pub async fn sync() -> Result<()> {
    sync::sync().await?;
    load()
}

pub fn init() -> Result<()> {
    if !is_tmux_running() {
        eprintln!("ERROR: Tmux is not running");
        exit(1);
    }

    init::init()
}
