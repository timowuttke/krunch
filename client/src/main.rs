use crate::krunch::Krunch;
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::{thread, time};

mod krunch;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create all files and kubernetes ressources necessary to run krunch
    Install,
    /// Remove all files and kubernetes ressources created by krunch
    Uninstall,
    /// Equivalent to "kubectl get"
    Get {
        #[clap(name = "kubectl_get_args")]
        kubectl_get_args: Vec<String>,
    },
    /// Equivalent to "kubectl delete"
    Delete {
        #[clap(name = "kubectl_delete_args")]
        kubectl_delete_args: Vec<String>,
    },
    /// Equivalent to "kubectl describe"
    Describe {
        #[clap(name = "kubectl_describe_args")]
        kubectl_describe_args: Vec<String>,
    },
    Run,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    let mut krunch = Krunch::new().await?;

    match &args.command {
        Command::Install => krunch.init().await?,

        Command::Get { kubectl_get_args } => {
            let kubectl_command = kubectl_get_args
                .iter()
                .fold("kubectl get".to_string(), |acc, x| format!("{} {}", acc, x));
            krunch.execute_pod_command(kubectl_command).await?;
        }

        Command::Run => {
            krunch.mount_current_path().await?;
            krunch
                .execute_pod_command("skaffold run -n default".to_string())
                .await?;
            krunch.unmount().await?;
        }

        _ => todo!(),
    }

    Ok(())
}
