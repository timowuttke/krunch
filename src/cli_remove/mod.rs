use crate::cli_remove::remove_tls_secret::remove_tls_secret;
use anyhow::Result;
use std::io;
use std::io::Write;

mod remove_tls_secret;

pub async fn cli_remove() -> Result<()> {
    print!("{:<30}", "creating TLS secret");
    io::stdout().flush().unwrap();
    if let Err(err) = remove_tls_secret().await {
        println!("{}", err)
    };

    Ok(())
}
