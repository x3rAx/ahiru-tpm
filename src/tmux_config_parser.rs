use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use pest::{Parser, iterators::Pair};
use pest_derive::Parser;

use crate::spec::Spec;

#[derive(Debug)]
pub enum ConfigDirective {
    PluginSpec(Spec),
    Source(PathBuf),
}

#[derive(Parser)]
#[grammar = "tmux-config.pest"]
struct TmuxConfigParser;

mod source_path {
    use std::path::{Path, PathBuf};

    use anyhow::{Context, Result};
    use pest::Parser;
    use pest_derive::Parser;

    use crate::tmux;

    #[derive(Parser)]
    #[grammar = "source-path.pest"]
    struct SourcePathParser;

    pub fn parse(path: &str, config_dir: &Path) -> Result<PathBuf> {
        let pairs = SourcePathParser::parse(Rule::source_path, path)
            .context(format!("Failed to parse source path: {path}"))?;

        // Get the start path of tmux so that relative `source` directives can be resolved.
        let mut target = tmux::get_start_path()?;

        for pair in pairs {
            match pair.as_rule() {
                Rule::root_dir => target = PathBuf::from("/"),
                Rule::config_dir => target = PathBuf::from(config_dir),
                Rule::path => target.push(pair.as_str().trim_start_matches('/')),

                Rule::EOI => (),

                Rule::source_path => unreachable!(),
            }
        }

        Ok(target)
    }
}

pub fn parse(config: &Path) -> Result<Vec<ConfigDirective>> {
    let config_dir = config.parent().expect("Config file should have a parent");
    let config = fs::read_to_string(config)
        .context(format!("Failed reading config file: {}", config.display()))?;

    let pairs = TmuxConfigParser::parse(Rule::config, &config)
        .context(format!("Failed to parse config file: {config}"))?;

    let directives: Vec<ConfigDirective> = pairs
        .filter_map(|pair| match pair.as_rule() {
            Rule::plugin_spec => Some(parse_plugin_spec_rule(pair)),
            Rule::source => Some(parse_source_rule(pair, config_dir)),

            Rule::other | Rule::EOI => None,

            Rule::WHITESPACE
            | Rule::COMMENT
            | Rule::config
            | Rule::directive
            | Rule::newline
            | Rule::set_option
            | Rule::source_file
            | Rule::source_file_flags
            | Rule::quoted_string
            | Rule::double_quoted_string
            | Rule::single_quoted_string
            | Rule::quoted_inner => {
                unreachable!("Unexpected rule: {:?}", pair.as_rule());
            }
        })
        .collect::<Result<_>>()?;

    Ok(directives)
}

fn parse_plugin_spec_rule(pair: Pair<'_, Rule>) -> Result<ConfigDirective> {
    let spec = parse_quoted_string(
        pair.into_inner()
            .find(|x| x.as_rule() == Rule::quoted_string)
            .context("`quoted_string` should exist inside `plugin_spec`")?,
    )
    .context("Failed to parse `quoted_string` of `plugin_spec`")?;
    Ok(ConfigDirective::PluginSpec(
        spec.as_str()
            .try_into()
            .with_context(|| format!("Failed to parse plugin spec: {spec}"))?,
    ))
}

fn parse_source_rule(pair: Pair<'_, Rule>, config_dir: &Path) -> Result<ConfigDirective> {
    let path = parse_quoted_string(
        pair.into_inner()
            .find(|x| x.as_rule() == Rule::quoted_string)
            .context("`quoted_string` should exist inside `source`")?,
    )
    .context("Failed to parse `quoted_string` of `source`")?;

    let path = source_path::parse(&path, config_dir)
        .with_context(|| format!("Failed parsing source_path: {path}"))?;

    Ok(ConfigDirective::Source(path.to_owned()))
}

fn parse_quoted_string(pair: Pair<'_, Rule>) -> Result<String> {
    let quoted_inner = pair
        .into_inner()
        .next()
        .context("`quoted_string` should have a child")?;

    Ok(match quoted_inner.as_rule() {
        Rule::double_quoted_string => quoted_inner
            .into_inner()
            .next()
            .context("`double_quoted_string` should have a child")?
            .as_str()
            .to_owned()
            .replace("\\\"", "\""),

        Rule::single_quoted_string => quoted_inner
            .into_inner()
            .next()
            .context("`single_quoted_string` should have a child")?
            .as_str()
            .to_owned(),

        _ => unreachable!(
            "Unexpected rule in `quoted_string`: {:?}",
            quoted_inner.as_rule()
        ),
    })
}
