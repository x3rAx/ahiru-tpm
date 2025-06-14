use anyhow::Result;

pub async fn sync() -> Result<()> {
    super::install::install().await?;
    super::clean::clean()?;
    super::update::update_all()?;
    Ok(())
}
