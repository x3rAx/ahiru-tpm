use std::collections::HashMap;

use anyhow::{Context, Result, anyhow};
use derive_builder::Builder;
use getset::Getters;
use log::{error, warn};
use pest::{Parser, iterators::Pairs};
use pest_derive::Parser;

use crate::{
    attribute::Attribute,
    repo_url::{RepoUrl, UrlAlias},
};

#[derive(Builder, Debug, Getters, PartialEq, Clone)]
pub struct Spec {
    #[getset(get = "pub")]
    name: String,

    #[getset(get = "pub")]
    url: RepoUrl,

    #[builder(default, setter(strip_option))]
    branch: Option<String>,

    #[getset(get = "pub")]
    #[builder(default, setter(each = "attribute"))]
    attributes: HashMap<Attribute, String>,
}

impl Spec {
    pub fn try_from_legacy(value: &str) -> Result<Spec> {
        if value.contains(';') {
            Err(anyhow!(
                "Attributes are not supported in legacy plugin definition using `@tpm_plugins`"
            )
            .context(format!("Failed to parse: {value}")))
        } else {
            // Force non-parallel loading for legacy plugins
            let value = value.to_owned() + "; parallel=false";
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
    let mut builder = SpecBuilder::default();

    for pair in SpecParser::parse(Rule::spec, value)? {
        parse_spec_pair(&mut builder, pair)?;
    }

    builder
        .build()
        .context(format!("Building plugin spec from: {value}"))
}

fn parse_spec_pair(builder: &mut SpecBuilder, pair: pest::iterators::Pair<'_, Rule>) -> Result<()> {
    match pair.as_rule() {
        Rule::url => {
            parse_url(builder, pair)?;
        }

        Rule::attribute => {
            parse_attribute(builder, pair.into_inner()).context("Failed to parse attribute")?;
        }

        Rule::EOI => return Ok(()),

        Rule::WHITESPACE
        | Rule::spec
        | Rule::short_url
        | Rule::alias_url
        | Rule::prefix
        | Rule::prefix_codeberg
        | Rule::prefix_github
        | Rule::prefix_gitlab
        | Rule::prefix_bitbucket
        | Rule::full_url
        | Rule::branch
        | Rule::user
        | Rule::repo
        | Rule::ident
        | Rule::attributes
        | Rule::attr_sep
        | Rule::attr_key
        | Rule::attr_val
        | Rule::quoted_string
        | Rule::single_quoted_string
        | Rule::double_quoted_string
        | Rule::quoted_inner
        | Rule::unquoted_val => {
            error!("Unexpected rule: {:#?}", pair.as_rule());
            unreachable!();
        }
    };
    Ok(())
}

fn parse_url(
    builder: &mut SpecBuilder,
    pair: pest::iterators::Pair<'_, Rule>,
) -> Result<(), anyhow::Error> {
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::short_url => {
                parse_short_url(builder, pair)?;
            }
            Rule::alias_url => {
                parse_alias_url(builder, pair)?;
            }
            Rule::full_url => {
                parse_full_url(builder, pair)?;
            }
            Rule::branch => {
                builder.branch(pair.as_str().to_string());
            }

            _ => {
                error!("Unexpected rule in `url`: {:#?}", pair.as_rule());
                unreachable!();
            }
        }
    }

    Ok(())
}

fn parse_short_url(
    builder: &mut SpecBuilder,
    pair: pest::iterators::Pair<'_, Rule>,
) -> Result<(), anyhow::Error> {
    builder.url(RepoUrl::Short(pair.as_str().to_string()));
    builder.name(
        pair.into_inner()
            .find(|x| x.as_rule() == Rule::repo)
            .context("`short_url` should contain `repo`")?
            .as_str()
            .to_string(),
    );
    Ok(())
}

fn parse_alias_url(
    builder: &mut SpecBuilder,
    pair: pest::iterators::Pair<'_, Rule>,
) -> Result<(), anyhow::Error> {
    let mut inner = pair.into_inner();
    let prefix = inner
        .find(|p| p.as_rule() == Rule::prefix)
        .context("`alias_url` should contain `prefix`")?
        .into_inner()
        .next()
        .context("`prefix` should have a child")?;
    let url_alias = match prefix.as_rule() {
        Rule::prefix_codeberg => UrlAlias::Codeberg,
        Rule::prefix_github => UrlAlias::GitHub,
        Rule::prefix_gitlab => UrlAlias::GitLab,
        Rule::prefix_bitbucket => UrlAlias::BitBucket,

        _ => {
            error!("Unexpected rule in `alias_url`: {:?}", prefix.as_rule());
            unreachable!();
        }
    };
    let short_url = inner
        .find(|p| p.as_rule() == Rule::short_url)
        .context("`alias_url` should contain `short_url`")?;
    builder.url(RepoUrl::Alias(url_alias, short_url.as_str().to_string()));
    builder.name(
        short_url
            .into_inner()
            .find(|x| x.as_rule() == Rule::repo)
            .context("`short_url` should contain `repo`")?
            .as_str()
            .to_string(),
    );
    Ok(())
}

fn parse_full_url(
    builder: &mut SpecBuilder,
    pair: pest::iterators::Pair<'_, Rule>,
) -> Result<(), anyhow::Error> {
    builder.url(RepoUrl::Full(pair.as_str().to_string()));
    builder.name(
        pair.into_inner()
            .find(|x| x.as_rule() == Rule::repo)
            .context("`full_url` should contain `repo`")?
            .as_str()
            .to_string(),
    );
    Ok(())
}

fn parse_attribute(builder: &mut SpecBuilder, mut attribute_pairs: Pairs<'_, Rule>) -> Result<()> {
    let key = attribute_pairs
        .find(|p| p.as_rule() == Rule::attr_key)
        .context("Attribute pairs should have an attribute key")?
        .as_str()
        .to_owned();

    let val_pair = attribute_pairs
        .find(|p| p.as_rule() == Rule::attr_val)
        .context("Attribute pairs should have an attribute value")?
        .into_inner()
        .next()
        .context("`attr_val` should have a child")?;

    let val = match val_pair.as_rule() {
        Rule::unquoted_val => val_pair.as_str().to_owned(),

        Rule::double_quoted_string => val_pair
            .into_inner()
            .next()
            .context("`double_quoted_string` should have a child")?
            .as_str()
            .to_owned()
            .replace("\\\"", "\""),

        Rule::single_quoted_string => val_pair
            .into_inner()
            .next()
            .context("`single_quoted_string` should have a child")?
            .as_str()
            .to_owned(),

        _ => unreachable!(),
    };

    if let Ok(key) = Attribute::try_from(key.as_ref()) {
        builder.attribute((key, val));
    } else {
        warn!("Invalid attribute name: {key}");
    }

    Ok(())
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
