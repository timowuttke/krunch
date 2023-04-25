use anyhow::Result;
use kube::Client;

mod build_image;
mod init;
mod krunch;

pub struct Krunch {
    client: Client,
}

#[tokio::main]
async fn main() -> Result<()> {
    // log level is controlled by RUST_LOG env variable, RUST_LOG="error/warn/info/debug". Default is error
    env_logger::init();

    let krunch = Krunch::new().await?;

    // Krunch::execute_host_command("echo test123")?;

    krunch.create_namespace().await?;
    krunch.create_service_account().await?;
    krunch.create_cluster_role_binding().await?;
    krunch.create_deployment().await?;
    krunch.verify_pod_is_healthy().await?;

    // let command = krunch.create_command()?;
    // krunch.execute_generic_command(command).await?;

    Ok(())
}
