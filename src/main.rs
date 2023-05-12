use crate::cli_install::cli_install;
use crate::cli_remove::cli_remove;
use crate::cli_version::cli_version;
use crate::shared::get_minikube_client;
use anyhow::Result;
use clap::{Parser, Subcommand};

mod cli_install;
mod cli_remove;
mod cli_version;
mod shared;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Setup a minikube based dev setup
    Install,
    /// Remove all files and configuration created by krunch
    Remove,
    /// Display version information
    Version,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    match &args.command {
        Commands::Install => {
            get_minikube_client().await?;
            cli_install().await?
        }
        Commands::Remove => {
            get_minikube_client().await?;
            cli_remove().await?
        }
        Commands::Version => cli_version(),
    }

    Ok(())
}
