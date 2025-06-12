use std::fmt::{self, Display};

use strum::{Display, EnumString};
use url::Url;

#[derive(Debug, PartialEq, Clone)]
pub enum RepoUrl {
    Short(String),
    Full(String),
    Alias(UrlAlias, String),
}

#[derive(EnumString, Debug, PartialEq, Clone, Display)]
pub enum UrlAlias {
    #[strum(serialize = "codeberg")]
    Codeberg,

    #[strum(serialize = "github")]
    GitHub,

    #[strum(serialize = "gitlab")]
    GitLab,

    #[strum(serialize = "bitbucket")]
    BitBucket,
}

impl Display for RepoUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RepoUrl::*;

        let url = match self {
            Short(url) => url,
            Full(url) => url,
            Alias(alias, url) => &format!("{}:{}", alias, url).to_string(),
        };

        write!(f, "{url}")
    }
}

impl From<&RepoUrl> for Url {
    fn from(value: &RepoUrl) -> Self {
        use RepoUrl::*;
        use UrlAlias::*;

        match value {
            Short(url) => Url::parse(&format!("https://github.com/{url}.git")),
            Full(url) => Url::parse(url),
            Alias(Codeberg, url) => Url::parse(&format!("https://codeberg.org/{url}.git")),
            Alias(GitHub, url) => Url::parse(&format!("https://github.com/{url}.git")),
            Alias(GitLab, url) => Url::parse(&format!("https://gitlab.com/{url}.git")),
            Alias(BitBucket, url) => Url::parse(&format!("https://gitbucket.org/{url}.git")),
        }
        .expect("Url should be valid")
    }
}

impl From<RepoUrl> for Url {
    fn from(value: RepoUrl) -> Self {
        Url::from(&value)
    }
}
