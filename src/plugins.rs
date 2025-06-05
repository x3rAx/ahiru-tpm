use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use anyhow::{Context, Result, anyhow};
use pest::Parser;
use pest_derive::Parser;
use regex::Regex;

use crate::{spec::Spec, tmux};

#[derive(Parser)]
#[grammar = "quoted-string.pest"]
struct QuotedParser;

pub fn load_specs() -> Result<Vec<Spec>> {
    let config_path =
        tmux::get_config_path().ok_or_else(|| anyhow!("Failed to find tmux config file"))?;

    load_specs_from_config(&config_path)?
        .into_iter()
        .map(Spec::try_from)
        .collect::<Result<_, _>>()
}

fn load_specs_from_config(config_path: &Path) -> Result<Vec<String>> {
    let file = File::open(config_path)?;
    let reader = BufReader::new(file);
    let re = Regex::new(r#"^\s*set(-option)?\s+-g\s+@plugin\s+(?P<spec>.*)"#)?;

    reader
        .lines()
        .filter_map(|line| match line {
            Ok(l) => re
                .captures(&l)
                .and_then(|caps| caps.name("spec").map(|m| Ok(m.as_str().to_string()))),
            Err(e) => Some(Err(e)),
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|s| match QuotedParser::parse(Rule::quoted_string, &s) {
            Ok(mut pairs) => Ok(pairs
                .next()
                .expect("The first pair should always exist")
                .into_inner()
                .next()
                .expect("The single or double quoted string should always exist")
                .into_inner()
                .next()
                .expect("The single or double inner should always exist")
                .as_str()
                .to_string()),
            Err(e) => Err(e).context("Failed to parse plugins from tmux config"),
        })
        .collect::<Result<_, _>>()
}
