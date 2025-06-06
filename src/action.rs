use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum Action {
    Install,
    Load,
}
