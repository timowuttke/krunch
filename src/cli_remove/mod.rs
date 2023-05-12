use crate::cli_remove::remove_binaries::remove_binaries;
use crate::cli_remove::remove_dns_for_minikube::remove_dns_for_minikube;
use crate::cli_remove::remove_tls_secret::remove_tls_secret;
use anyhow::Result;
use remove_environment_entries::remove_environment_entries;
use std::io;
use std::io::Write;

mod remove_binaries;
mod remove_dns_for_minikube;
mod remove_environment_entries;
mod remove_tls_secret;

pub async fn cli_remove() -> Result<()> {
    print!("{:<35}", "deleting environment entries");
    io::stdout().flush().unwrap();
    if let Err(err) = remove_environment_entries() {
        println!("{}", err)
    };

    print!("{:<35}", "deleting DNS entry");
    io::stdout().flush().unwrap();
    if let Err(err) = remove_dns_for_minikube() {
        println!("{}", err)
    };

    print!("{:<35}", "deleting local CA");
    io::stdout().flush().unwrap();
    if let Err(err) = remove_tls_secret().await {
        println!("{}", err)
    };

    print!("{:<35}", "deleting downloaded files");
    io::stdout().flush().unwrap();
    if let Err(err) = remove_binaries() {
        println!("{}", err)
    };

    Ok(())
}
