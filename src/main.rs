use anyhow::Result;
use tpm_rs::tmux::get_option;

fn main() -> Result<()> {
    let tpm_async = get_option("@tpm_async").unwrap_or("".into()) == "true";

    println!("@tpm_async: {}", tpm_async);

    Ok(())
}
