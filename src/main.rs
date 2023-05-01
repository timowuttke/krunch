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
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    let krunch = Krunch::new().await?;

    match &args.command {
        Commands::Init => krunch.init().await?,
    }

    Ok(())
}
