use crate::init::cli_init;
use anyhow::Result;
use clap::{Parser, Subcommand};

mod r#const;
mod init;

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

    match &args.command {
        Commands::Init => cli_init().await?,
    }

    Ok(())
}
