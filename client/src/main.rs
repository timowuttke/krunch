use anyhow::Result;
use clap::{Parser, Subcommand};
use kube::Client;
use std::io;
use std::io::Write;

mod build_image;
mod init;
mod krunch;

pub struct Krunch {
    client: Client,
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// setup Krunch for use
    Init,
}

const NAMESPACE: &'static str = "krunch";
const SERVICE_ACCOUNT: &'static str = "krunch";
const CLUSTER_ROLE_BINDING: &'static str = "krunch-gets-cluster-admin";
const DEPLOYMENT: &'static str = "krunch";
const IMAGE: &'static str = "timowuttke/krunch:v1";

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    let krunch = Krunch::new().await?;

    krunch.init().await?;

    // Krunch::execute_host_command("echo test123")?;
    // let command = krunch.create_command()?;
    // krunch.execute_generic_command(command).await?;

    Ok(())
}
