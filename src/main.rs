use anyhow::Result;
use tpm_rs::{
    action::Action,
    plugins::{self},
};

fn main() -> Result<()> {
    // Get first argument
    let action = Action::try_from(std::env::args().nth(1))?;

    println!("{action:?}");

    match action {
        Action::Install => plugins::install(),
    }
}
