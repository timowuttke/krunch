use crate::cli_remove::remove_tls_secret::remove_tls_secret;
use anyhow::Result;
use remove_environment_entries::remove_environment_entries;
use std::io;
use std::io::Write;

mod remove_environment_entries;
mod remove_tls_secret;

pub async fn cli_remove() -> Result<()> {
    print!("{:<30}", "deleting environment entries");
    io::stdout().flush().unwrap();
    if let Err(err) = remove_environment_entries() {
        println!("{}", err)
    };

    print!("{:<30}", "deleting TLS secret");
    io::stdout().flush().unwrap();
    if let Err(err) = remove_tls_secret().await {
        println!("{}", err)
    };

    Ok(())
}
