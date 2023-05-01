use crate::krunch::Krunch;
use anyhow::Result;
use clap::{Parser, Subcommand};

mod downloads;
mod krunch;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// download common tools and create local CA to access minikube over https
    Init,
    Danger,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    match &args.command {
        Commands::Init => {
            let krunch = Krunch::new().await?;
            krunch.init().await?
        }
        Commands::Danger => Krunch::add_bin_folder_to_path().await?,
    }

    Ok(())
}
