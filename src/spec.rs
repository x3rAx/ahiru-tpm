use anyhow::{Result, anyhow};
use getset::Getters;
use pest::Parser;
use pest_derive::Parser;

#[derive(Debug, Getters)]
pub struct Spec {
    #[getset(get = "pub")]
    name: String,

    #[getset(get = "pub")]
    url: String,

    #[getset(get = "pub")]
    branch: Option<String>,
}

#[derive(Parser)]
#[grammar = "spec.pest"]
struct SpecParser;

impl TryFrom<String> for Spec {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self> {
        if value.is_empty() {
            Err(anyhow!("Plugin spec must not be empty"))
        } else if value.contains(';') {
            Err(anyhow!("Attributes are not supported yet")
                .context(format!("Failed to parse: {value}")))
        } else {
            let parsed = SpecParser::parse(Rule::spec, &value)?;

            let mut name = None;
            let mut url = None;
            let mut branch = None;

            for pair in parsed {
                match pair.as_rule() {
                    Rule::short_url => {
                        url = Some(pair.as_str().to_string());
                        name = pair
                            .into_inner()
                            .find(|x| x.as_rule() == Rule::repo)
                            .map(|r| r.as_str().to_string());
                    }
                    Rule::branch => branch = Some(pair.as_str().to_string()),
                    _ => (),
                };
            }

            Ok(Spec {
                name: name.ok_or_else(|| anyhow!("Failed to get plugin name"))?,
                url: url.ok_or_else(|| anyhow!("Failed to get plugin url"))?,
                branch,
            })
        }
    }
}
