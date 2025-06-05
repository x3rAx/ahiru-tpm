use std::fmt::{self, Display};

#[derive(Debug, PartialEq)]
pub enum RepoUrl {
    Short(String),
}

impl Display for RepoUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RepoUrl::*;

        let url = match self {
            Short(url) => format!("https://github.com/{url}.git"),
        };

        write!(f, "{url}")
    }
}
