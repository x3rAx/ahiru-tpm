use anyhow::{Result, anyhow};
use getset::Getters;
use pest::Parser;
use pest_derive::Parser;

use crate::repo_url::RepoUrl;

#[derive(Debug, Getters, PartialEq, Clone)]
pub struct Spec {
    #[getset(get = "pub")]
    name: String,

    #[getset(get = "pub")]
    url: RepoUrl,

    branch: Option<String>,
}

impl Spec {
    pub fn try_from_url(value: &str) -> Result<Spec> {
        println!("try_from_url: {value}");
        if value.is_empty() {
            Err(anyhow!("Plugin URL must not be empty"))
        } else if value.contains(';') {
            println!("contains ';'");
            Err(anyhow!(
                "Attributes are not supported in legacy plugin definition using `@tpm_plugins`"
            )
            .context(format!("Failed to parse: {value}")))
        } else {
            parse_spec(value)
        }
    }

    pub fn branch(&self) -> Option<&str> {
        self.branch.as_deref()
    }
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

    for pair in SpecParser::parse(Rule::spec, value)? {
        match pair.as_rule() {
            Rule::short_url => {
                url = Some(RepoUrl::Short(pair.as_str().to_string()));
                name = pair
                    .into_inner()
                    .find(|x| x.as_rule() == Rule::repo)
                    .map(|r| r.as_str().to_string());
            }
            Rule::branch => branch = Some(pair.as_str().to_string()),
            Rule::full_url => {
                url = Some(RepoUrl::Full(pair.as_str().to_string()));
                name = pair
                    .into_inner()
                    .find(|x| x.as_rule() == Rule::repo)
                    .map(|r| r.as_str().to_string());
            }
            Rule::EOI => break,
            Rule::spec | Rule::url | Rule::user | Rule::repo | Rule::ident => unreachable!(),
        };
    }

    Ok(Spec {
        name: name.ok_or_else(|| anyhow!("Failed to get plugin name from spec"))?,
        url: url.ok_or_else(|| anyhow!("Failed to get plugin url from spec"))?,
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
            url: RepoUrl::Short("user_name/repo-name".into()),
            branch: None,
        };
        assert_eq!(Spec::try_from(value).unwrap(), expected_spec);
    }

    #[test]
    fn test_parse_short_url_with_branch() {
        let value = "user_name/repo-name#branch/name";
        let expected_spec = Spec {
            name: "repo-name".into(),
            url: RepoUrl::Short("user_name/repo-name".into()),
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
