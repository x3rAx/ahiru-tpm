use std::fmt::Display;

use url::Url;

use crate::spec::Spec;

pub struct Plugin {
    spec: Spec,
}

impl Plugin {
    pub fn url(&self) -> Url {
        self.spec.url().into()
    }

    pub fn name(&self) -> &str {
        self.spec.name()
    }
}

impl From<Spec> for Plugin {
    fn from(spec: Spec) -> Self {
        Plugin { spec }
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
