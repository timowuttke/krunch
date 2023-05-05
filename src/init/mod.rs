use crate::init::commands::install_local_ca;
use crate::init::downloads::download_all;
use crate::init::environment::{add_bin_folder_to_path, point_docker_to_minikube};
use crate::init::k8s::{enabling_ingress_addon, install_tls_secret};
use anyhow::Result;
use std::io;
use std::io::Write;

pub mod commands;
mod downloads;
mod environment;
mod k8s;
mod urls;

pub async fn cli_init() -> Result<()> {
    download_all().await?;

    print!("{:30}", "adding tools to PATH");
    io::stdout().flush().unwrap();

    if let Err(err) = add_bin_folder_to_path().await {
        println!("failed: {}", err)
    }

    print!("{:30}", "point docker cli to minikube");
    io::stdout().flush().unwrap();

    if let Err(err) = point_docker_to_minikube().await {
        println!("failed: {}", err)
    }

    print!("{:<30}", "creating TLS secret");
    io::stdout().flush().unwrap();
    install_local_ca()?;
    install_tls_secret().await?;

    print!("{:<30}", "enabling ingress addon");
    io::stdout().flush().unwrap();
    enabling_ingress_addon()?;

    Ok(())
}
