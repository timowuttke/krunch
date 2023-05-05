use crate::init::cli_init;
use crate::version::cli_version;
use anyhow::Result;
use clap::{Parser, Subcommand};

mod r#const;
mod init;
mod version;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// create a minikube base dev setup
    Init,
    /// display krunch version
    Version,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    match &args.command {
        Commands::Init => cli_init().await?,
        Commands::Version => cli_version(),
    }

    Ok(())
}
