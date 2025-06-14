use anyhow::Result;

pub fn sync() -> Result<()> {
    super::install::install()?;
    super::clean::clean()?;
    super::update::update_all()?;
    Ok(())
}
