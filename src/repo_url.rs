use std::fmt::{self, Display};

use url::Url;

#[derive(Debug, PartialEq)]
pub enum RepoUrl {
    Short(String),
}

impl Display for RepoUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RepoUrl::*;

        let Short(url) = self;

        write!(f, "{url}")
    }
}

impl From<&RepoUrl> for Url {
    fn from(value: &RepoUrl) -> Self {
        use RepoUrl::*;

        match value {
            Short(url) => Url::parse(&format!("https://github.com/{url}.git")),
        }
        .expect("Url should be valid")
    }
}

impl From<RepoUrl> for Url {
    fn from(value: RepoUrl) -> Self {
        Url::from(&value)
    }
}
