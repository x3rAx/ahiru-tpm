use std::{fs, path::Path};

use anyhow::{Context, Result};
use pest::{Parser, iterators::Pair};
use pest_derive::Parser;

use crate::spec::Spec;

#[derive(Debug)]
pub enum ConfigDirective {
    PluginSpec(Spec),
}

#[derive(Parser)]
#[grammar = "tmux-config.pest"]
struct TmuxConfigParser;

pub fn parse(config: &Path) -> Result<Vec<ConfigDirective>> {
    let config = fs::read_to_string(config)
        .context(format!("Failed reading config file: {}", config.display()))?;

    let pairs = TmuxConfigParser::parse(Rule::config, &config)?;

    let directives: Vec<ConfigDirective> = pairs
        .filter_map(|pair| match pair.as_rule() {
            Rule::plugin_spec => Some(parse_plugin_spec_rule(pair)),

            Rule::EOI => None,
            _ => unreachable!(),
        })
        .collect::<Result<_>>()?;

    println!("{directives:?}");
    Ok(directives)
}

fn parse_plugin_spec_rule(pair: Pair<'_, Rule>) -> Result<ConfigDirective> {
    let spec = parse_quoted_string(
        pair.into_inner()
            .find(|x| x.as_rule() == Rule::quoted_string)
            .expect("`quoted_string` should exist inside `plugin_spec`"),
    );
    Ok(ConfigDirective::PluginSpec(spec.try_into()?))
}

fn parse_quoted_string(pair: Pair<'_, Rule>) -> &str {
    pair.into_inner()
        .find(|x| [Rule::double_inner, Rule::single_inner].contains(&x.as_rule()))
        .expect("`double_inner` or single_inner should exist inside `quoted_string`")
        .as_str()
}
