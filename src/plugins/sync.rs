use anyhow::Result;

pub fn sync_cmd() -> Result<()> {
    sync()?;
    super::load::load()?;

    println!();
    println!("==> Done syncing plugins.");
    println!("==> Plugins have been reloaded.");

    Ok(())
}

pub fn sync() -> Result<()> {
    super::install::install()?;
    super::clean::clean_cmd()?;
    super::update::update_all()?;
    Ok(())
}
