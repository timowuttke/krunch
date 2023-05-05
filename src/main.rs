use crate::krunch::Krunch;
use anyhow::Result;
use clap::{Parser, Subcommand};

mod init;
mod krunch;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// create a minikube base dev setup
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
