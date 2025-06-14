use std::{fmt::Display, path::PathBuf};

use once_cell::sync::OnceCell;
use url::Url;

use crate::{attribute::Attribute, spec::Spec, tmux};

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
        write!(f, "{} ({}", self.spec.name(), self.spec.url())?;
        if let Some(branch) = self.spec.branch() {
            write!(f, "#{branch}")?;
        };
        write!(f, ")")
    }
}
