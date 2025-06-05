use anyhow::{Result, anyhow};

#[derive(Debug)]
pub struct Spec {
    pub plugin: String,
}

impl TryFrom<String> for Spec {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self> {
        if value.is_empty() {
            Err(anyhow!("String is empty"))
        } else if value.contains(';') {
            Err(anyhow!("Attributes are not supported yet")
                .context(format!("Failed to parse: {value}")))
        } else {
            Ok(Spec { plugin: value })
        }
    }
}
