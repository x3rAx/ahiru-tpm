#[derive(Debug)]
pub enum Action {
    Install,
}

fn parse_action(action: &str) -> Result<Action, anyhow::Error> {
    match action {
        "install" => Ok(Action::Install),
        _ => Err(anyhow::anyhow!("Unknown action: {action}")),
    }
}

impl TryFrom<&str> for Action {
    type Error = anyhow::Error;

    fn try_from(action: &str) -> Result<Self, Self::Error> {
        parse_action(action)
    }
}

impl TryFrom<String> for Action {
    type Error = anyhow::Error;

    fn try_from(action: String) -> Result<Self, Self::Error> {
        parse_action(&action)
    }
}

impl TryFrom<Option<&str>> for Action {
    type Error = anyhow::Error;

    fn try_from(action: Option<&str>) -> Result<Self, Self::Error> {
        match action {
            Some(s) => parse_action(s),
            None => Ok(Action::Install),
        }
    }
}

impl TryFrom<Option<String>> for Action {
    type Error = anyhow::Error;

    fn try_from(action: Option<String>) -> Result<Self, Self::Error> {
        Action::try_from(action.as_deref())
    }
}
