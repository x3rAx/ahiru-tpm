use std::{fmt::Display, path::PathBuf, sync::OnceLock};

use url::Url;

use crate::{spec::Spec, tmux};

pub struct Plugin {
    spec: Spec,
    path: OnceLock<Option<PathBuf>>,
}

impl Plugin {
    pub fn url(&self) -> Url {
        self.spec.url().into()
    }

    pub fn name(&self) -> &str {
        self.spec.name()
    }

    pub fn is_installed(&self) -> Option<bool> {
        self.path().map(|p| p.exists())
    }

    pub fn path(&self) -> Option<&PathBuf> {
        self.path
            .get_or_init(|| tmux::get_plugins_dir().map(|p| p.join(self.name())))
            .as_ref()
    }
}

impl From<Spec> for Plugin {
    fn from(spec: Spec) -> Self {
        Plugin {
            spec,
            path: OnceLock::new(),
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
