use anyhow::{Result, anyhow};
use getset::Getters;
use pest::Parser;
use pest_derive::Parser;

#[derive(Debug, Getters, PartialEq)]
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

impl TryFrom<&str> for Spec {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        if value.is_empty() {
            Err(anyhow!("Plugin spec must not be empty"))
        } else if value.contains(';') {
            Err(anyhow!("Attributes are not supported yet")
                .context(format!("Failed to parse: {value}")))
        } else {
            parse_spec(value)
        }
    }
}

impl TryFrom<String> for Spec {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self> {
        Self::try_from(value.as_ref())
    }
}

fn parse_spec(value: &str) -> std::result::Result<Spec, anyhow::Error> {
    let mut name = None;
    let mut url = None;
    let mut branch = None;

    for pair in SpecParser::parse(Rule::spec, &value)? {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_short_url() {
        let value = "user_name/repo-name";
        let expected_spec = Spec {
            name: "repo-name".into(),
            url: "user_name/repo-name".into(),
            branch: None,
        };
        assert_eq!(Spec::try_from(value).unwrap(), expected_spec);
    }

    #[test]
    fn test_parse_short_url_with_branch() {
        let value = "user_name/repo-name#branch/name";
        let expected_spec = Spec {
            name: "repo-name".into(),
            url: "user_name/repo-name".into(),
            branch: Some("branch/name".into()),
        };
        assert_eq!(Spec::try_from(value).unwrap(), expected_spec);
    }

    #[test]
    fn test_should_error_on_empty_value() {
        let value = "";
        let result = Spec::try_from(value);
        let err = result.unwrap_err();
        assert_eq!(format!("{err}"), "Plugin spec must not be empty");
    }
}
