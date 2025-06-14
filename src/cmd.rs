use anyhow::Result;

use crate::{
    args::{InstallArgs, UpdateArgs},
    plugins::{clean, install, load, sync, update},
};

pub fn install(args: InstallArgs) -> Result<()> {
    install::install()?;

    if args.load {
        load()?;
    }

    println!();
    println!("==> Done installing plugins.");
    if args.load {
        println!("==> Plugins have been reloaded.");
    }

    Ok(())
}

pub fn update(args: UpdateArgs) -> Result<()> {
    if args.all {
        update::update_all()?;
    } else {
        update::update_list(&args.names)?;
    }

    if args.load {
        load()?;
    }

    println!("==> Done updating plugins.");
    if args.load {
        println!("==> Plugins have been reloaded.");
    }

    Ok(())
}

pub fn load() -> Result<()> {
    load::load()?;

    println!();
    println!("==> Plugins have been reloaded.");
    Ok(())
}

pub fn clean() -> Result<()> {
    clean::clean()?;

    println!();
    println!("==> Done cleaning plugins.");

    Ok(())
}

pub fn sync() -> Result<()> {
    sync::sync()?;
    load::load()?;

    println!();
    println!("==> Done syncing plugins.");
    println!("==> Plugins have been reloaded.");

    Ok(())
}
