use crate::Krunch;
use anyhow::Result;
use std::io;
use std::io::Write;

mod commands;
mod downloads;
mod environment;
mod k8s;
mod urls;

impl Krunch {
    pub async fn init(&self) -> Result<()> {
        Krunch::download_all().await?;

        print!("{:30}", "adding tools to PATH");
        io::stdout().flush().unwrap();

        if let Err(err) = Self::add_bin_folder_to_path().await {
            println!("failed: {}", err)
        }

        print!("{:30}", "point docker cli to minikube");
        io::stdout().flush().unwrap();

        if let Err(err) = Self::point_docker_to_minikube().await {
            println!("failed: {}", err)
        }

        print!("{:<30}", "creating TLS secret");
        io::stdout().flush().unwrap();
        Self::install_local_ca()?;
        self.install_tls_secret().await?;

        print!("{:<30}", "enabling ingress addon");
        io::stdout().flush().unwrap();
        self.enabling_ingress_addon()?;

        Ok(())
    }
}
