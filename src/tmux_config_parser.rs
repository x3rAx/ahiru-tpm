use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use log::error;
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
                _ => unreachable!(),
            };
        }

        Ok(target)
    }
}

pub fn parse(config: &Path) -> Result<Vec<ConfigDirective>> {
    let config_dir = config.parent().expect("Config file should have a parent");
    let config = fs::read_to_string(config)
        .context(format!("Failed reading config file: {}", config.display()))?;

    let pairs = TmuxConfigParser::parse(Rule::config, &config)?;

    let directives: Vec<ConfigDirective> = pairs
        .filter_map(|pair| match pair.as_rule() {
            Rule::plugin_spec => Some(parse_plugin_spec_rule(pair)),
            Rule::source => Some(parse_source_rule(pair, config_dir)),

            Rule::other | Rule::EOI => None,
            _ => {
                error!("Unexpected rule: {:?}", pair.as_rule());
                unreachable!();
            }
        })
        .collect::<Result<_>>()?;

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

fn parse_source_rule(pair: Pair<'_, Rule>, config_dir: &Path) -> Result<ConfigDirective> {
    let path = parse_quoted_string(
        pair.into_inner()
            .find(|x| x.as_rule() == Rule::quoted_string)
            .expect("`quoted_string` should exist `inside source`"),
    );

    let path = source_path::parse(path, config_dir).context("Failed parsing source_path")?;

    Ok(ConfigDirective::Source(path.to_owned()))
}

fn parse_quoted_string(pair: Pair<'_, Rule>) -> &str {
    pair.into_inner()
        .find(|x| [Rule::double_inner, Rule::single_inner].contains(&x.as_rule()))
        .expect("`double_inner` or single_inner should exist inside `quoted_string`")
        .as_str()
}
