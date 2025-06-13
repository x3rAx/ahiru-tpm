use strum::{Display, EnumString};

#[derive(EnumString, Display, Debug, Eq, Hash, PartialEq, Clone)]
pub enum Attribute {
    #[strum(serialize = "alias")]
    Alias,

    #[strum(serialize = "parallel")]
    Parallel,
}
