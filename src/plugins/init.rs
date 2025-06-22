use anyhow::Result;

use crate::tmux;

use super::load;

pub fn init() -> Result<()> {
    tmux::setup_keymaps()?;
    load::load()?;

    Ok(())
}
