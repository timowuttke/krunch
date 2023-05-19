use crate::cli_install::bin_folder_to_path::add_bin_folder_to_path;
use crate::cli_install::create_ca_and_tls::create_ca_and_tls;
use crate::cli_install::dns_for_minikube::add_dns_for_minikube;
use crate::cli_install::docker_to_minikube::point_docker_to_minikube;
use crate::cli_install::download_binaries::download_all;
use crate::cli_install::enable_ingress::enable_ingress_addon_if_needed;
use crate::shared::should_continue_as_admin;
use anyhow::Result;
use std::io;
use std::io::Write;

mod bin_folder_to_path;
mod create_ca_and_tls;
mod dns_for_minikube;
mod docker_to_minikube;
mod download_binaries;
mod download_urls;
mod enable_ingress;
mod get_versions;

pub async fn cli_install() -> Result<()> {
    print!("{:<35}", "downloading tools");
    io::stdout().flush().unwrap();
    if let Err(err) = download_all().await {
        println!("{}", err)
    }

    print!("{:<35}", "adding tools to PATH");
    io::stdout().flush().unwrap();
    if let Err(err) = add_bin_folder_to_path().await {
        println!("{}", err)
    }

    print!("{:<35}", "point docker cli to minikube");
    io::stdout().flush().unwrap();
    if let Err(err) = point_docker_to_minikube().await {
        println!("{}", err)
    }

    print!("{:<35}", "enabling ingress addon");
    io::stdout().flush().unwrap();
    if let Err(err) = enable_ingress_addon_if_needed().await {
        println!("{}", err)
    };

    if should_continue_as_admin()? {
        print!("{:<35}", "creating DNS entry");
        io::stdout().flush().unwrap();
        if let Err(err) = add_dns_for_minikube() {
            println!("{}", err)
        }

        print!("{:<35}", "creating CA and TLS secret");
        io::stdout().flush().unwrap();
        if let Err(err) = create_ca_and_tls().await {
            println!("{}", err)
        };
    } else {
        println!("{:<35}{}", "creating DNS entry", "skipped (not admin)");
        println!(
            "{:<35}{}",
            "creating CA and TLS secret", "skipped (not admin)"
        );
    }

    Ok(())
}
