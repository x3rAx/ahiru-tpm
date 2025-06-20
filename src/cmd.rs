use anyhow::Result;

use crate::{
    args::{InstallArgs, UpdateArgs},
    plugins::{clean, install, load, sync, update},
};

pub async fn install(args: InstallArgs) -> Result<()> {
    install::install().await?;

    if args.load {
        load()?;
    }

    eprintln!();
    eprintln!("==> Done installing plugins.");
    if args.load {
        eprintln!("==> Plugins have been reloaded.");
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

    eprintln!("==> Done updating plugins.");
    if args.load {
        eprintln!("==> Plugins have been reloaded.");
    }

    Ok(())
}

pub fn load() -> Result<()> {
    load::load()?;

    eprintln!();
    eprintln!("==> Plugins have been reloaded.");
    Ok(())
}

pub fn clean() -> Result<()> {
    clean::clean()?;

    eprintln!();
    eprintln!("==> Done cleaning plugins.");

    Ok(())
}

pub async fn sync() -> Result<()> {
    sync::sync().await?;
    load::load()?;

    eprintln!();
    eprintln!("==> Done syncing plugins.");
    eprintln!("==> Plugins have been reloaded.");

    Ok(())
}
