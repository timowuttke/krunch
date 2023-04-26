use anyhow::Result;
use kube::Client;
use std::io;
use std::io::Write;

mod build_image;
mod init;
mod krunch;

pub struct Krunch {
    client: Client,
}

const NAMESPACE: &'static str = "krunch";
const SERVICE_ACCOUNT: &'static str = "krunch";
const CLUSTER_ROLE_BINDING: &'static str = "krunch-gets-cluster-admin";
const DEPLOYMENT: &'static str = "krunch";
const IMAGE: &'static str = "timowuttke/krunch:v1";

#[tokio::main]
async fn main() -> Result<()> {
    let krunch = Krunch::new().await?;

    krunch.init().await?;

    // Krunch::execute_host_command("echo test123")?;
    // let command = krunch.create_command()?;
    // krunch.execute_generic_command(command).await?;

    Ok(())
}

pub async fn println_async(text: &str) {
    println!("{}", text);
    io::stdout().flush().unwrap();
}

pub async fn print_async(text: &str) {
    print!("{}", text);
    io::stdout().flush().unwrap();
}
