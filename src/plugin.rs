use std::{fmt::Display, path::PathBuf};

use once_cell::sync::OnceCell;
use url::Url;

use crate::{attribute::Attribute, plugins, repo_url::RepoUrl, spec::Spec, tmux, utils};

pub struct Plugin {
    spec: Spec,
    // TODO: Once `std::cell::OnceCell::get_or_try_init` is stable replace `once_cell` crate with
    //       `std::cell`
    path: OnceCell<PathBuf>,
}

impl Plugin {
    pub fn url(&self) -> Url {
        self.spec.url().into()
    }

    pub fn repo_url(&self) -> &RepoUrl {
        self.spec.url()
    }

    pub fn name(&self) -> &str {
        self.spec
            .attributes()
            .get(&Attribute::Alias)
            .unwrap_or_else(|| self.spec.name())
    }

    pub fn is_installed(&self) -> bool {
        self.path().exists()
    }

    pub fn path(&self) -> &PathBuf {
        self.path
            .get_or_init(|| tmux::get_plugins_dir().join(self.name()))
    }

    pub fn branch(&self) -> Option<&str> {
        self.spec.branch()
    }

    pub fn parallel(&self) -> bool {
        self.spec
            .attributes()
            .get(&Attribute::Parallel)
            .and_then(|s| utils::parse_bool(s))
            .unwrap_or_else(plugins::do_parallel)
    }
}

impl From<Spec> for Plugin {
    fn from(spec: Spec) -> Self {
        Plugin {
            spec,
            path: OnceCell::new(),
        }
    }
}

impl Display for Plugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({}", self.name(), self.repo_url())?;
        if let Some(branch) = self.branch() {
            write!(f, "#{branch}")?;
        };
        write!(f, ")")
    }
}
