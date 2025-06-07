use std::fmt::{self, Display};

use url::Url;

#[derive(Debug, PartialEq, Clone)]
pub enum RepoUrl {
    Short(String),
    Full(String),
}

impl Display for RepoUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RepoUrl::*;

        let url = match self {
            Short(url) => url,
            Full(url) => url,
        };

        write!(f, "{url}")
    }
}

impl From<&RepoUrl> for Url {
    fn from(value: &RepoUrl) -> Self {
        use RepoUrl::*;

        match value {
            Short(url) => Url::parse(&format!("https://github.com/{url}.git")),
            Full(url) => Url::parse(url),
        }
        .expect("Url should be valid")
    }
}

impl From<RepoUrl> for Url {
    fn from(value: RepoUrl) -> Self {
        Url::from(&value)
    }
}
