use std::collections::HashMap;

use anyhow::{Result, anyhow};
use getset::Getters;
use log::{error, warn};
use pest::{Parser, iterators::Pairs};
use pest_derive::Parser;

use crate::{attribute::Attribute, repo_url::RepoUrl};

#[derive(Debug, Getters, PartialEq, Clone)]
pub struct Spec {
    #[getset(get = "pub")]
    name: String,

    #[getset(get = "pub")]
    url: RepoUrl,

    branch: Option<String>,

    #[getset(get = "pub")]
    attributes: HashMap<Attribute, String>,
}

impl Spec {
    pub fn try_from_url(value: &str) -> Result<Spec> {
        if value.contains(';') {
            Err(anyhow!(
                "Attributes are not supported in legacy plugin definition using `@tpm_plugins`"
            )
            .context(format!("Failed to parse: {value}")))
        } else {
            Spec::try_from(value)
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
    let mut attributes: HashMap<Attribute, String> = HashMap::new();

    for pair in SpecParser::parse(Rule::spec, value)? {
        match pair.as_rule() {
            Rule::url => {
                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::short_url => {
                            url = Some(RepoUrl::Short(pair.as_str().to_string()));
                            name = pair
                                .into_inner()
                                .find(|x| x.as_rule() == Rule::repo)
                                .map(|r| r.as_str().to_string());
                        }
                        Rule::full_url => {
                            url = Some(RepoUrl::Full(pair.as_str().to_string()));
                            name = pair
                                .into_inner()
                                .find(|x| x.as_rule() == Rule::repo)
                                .map(|r| r.as_str().to_string());
                        }
                        Rule::branch => branch = Some(pair.as_str().to_string()),

                        _ => {
                            error!("Unexpected rule in `url`: {:#?}", pair.as_rule());
                            unreachable!();
                        }
                    }
                }
            }

            Rule::attribute => parse_attribute(&mut attributes, pair.into_inner()),

            Rule::EOI => break,

            Rule::WHITESPACE
            | Rule::spec
            | Rule::short_url
            | Rule::full_url
            | Rule::branch
            | Rule::user
            | Rule::repo
            | Rule::ident
            | Rule::attributes
            | Rule::attr_key
            | Rule::attr_val
            | Rule::quoted_string
            | Rule::quoted_inner => {
                error!("Unexpected rule: {:#?}", pair.as_rule());
                unreachable!();
            }
        };
    }

    Ok(Spec {
        name: name.ok_or_else(|| anyhow!("Failed to get plugin name from spec: {value}"))?,
        url: url.ok_or_else(|| anyhow!("Failed to get plugin url from spec: {value}"))?,
        branch,
        attributes,
    })
}

fn parse_attribute(
    attributes: &mut HashMap<Attribute, String>,
    mut attribute_pairs: Pairs<'_, Rule>,
) {
    let key = attribute_pairs
        .find(|p| p.as_rule() == Rule::attr_key)
        .expect("Attribute pairs should have an attribute key")
        .as_str()
        .to_owned();
    let val = attribute_pairs
        .find(|p| p.as_rule() == Rule::attr_val)
        .expect("Attribute pairs should have an attribute value")
        .as_str()
        .to_owned();

    if let Ok(key) = Attribute::try_from(key.as_ref()) {
        attributes.insert(key, val);
    } else {
        warn!("Invalid attribute name: {key}");
    }
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
            attributes: HashMap::new(),
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
            attributes: HashMap::new(),
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
