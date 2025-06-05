use anyhow::{Result, anyhow};
use getset::Getters;

#[derive(Debug, Getters)]
pub struct Spec {
    #[getset(get = "pub")]
    plugin: String,
}

impl TryFrom<String> for Spec {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self> {
        if value.is_empty() {
            Err(anyhow!("Plugin spec must not be empty"))
        } else if value.contains(';') {
            Err(anyhow!("Attributes are not supported yet")
                .context(format!("Failed to parse: {value}")))
        } else {
            Ok(Spec { plugin: value })
        }
    }
}
