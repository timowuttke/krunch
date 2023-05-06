use crate::cli_init::bin_folder_to_path::add_bin_folder_to_path;
use crate::cli_init::create_tls_secret::create_ca_and_install_tls_in_cluster;
use crate::cli_init::docker_to_minikube::point_docker_to_minikube;
use crate::cli_init::download_binaries::download_all;
use crate::cli_init::enable_ingress::enable_ingress_addon_if_needed;
use anyhow::Result;
use std::io;
use std::io::Write;

mod bin_folder_to_path;
pub mod create_tls_secret;
mod docker_to_minikube;
mod download_binaries;
mod enable_ingress;
mod urls;

pub async fn cli_init() -> Result<()> {
    download_all().await?;

    print!("{:30}", "adding tools to PATH");
    io::stdout().flush().unwrap();
    if let Err(err) = add_bin_folder_to_path().await {
        println!("{}", err)
    }

    print!("{:30}", "point docker cli to minikube");
    io::stdout().flush().unwrap();
    if let Err(err) = point_docker_to_minikube().await {
        println!("{}", err)
    }

    print!("{:<30}", "creating TLS secret");
    io::stdout().flush().unwrap();
    if let Err(err) = create_ca_and_install_tls_in_cluster().await {
        println!("{}", err)
    };

    print!("{:<30}", "enabling ingress addon");
    io::stdout().flush().unwrap();
    if let Err(err) = enable_ingress_addon_if_needed() {
        println!("{}", err)
    };

    Ok(())
}
