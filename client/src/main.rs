use anyhow::Result;
use crate::krunch::Krunch;

mod krunch;

#[tokio::main]
async fn main() -> Result<()> {
    // log level is controlled by RUST_LOG env variable, RUST_LOG="error/warn/info/debug". Default is error
    env_logger::init();

    let krunch = Krunch::new().await?;
    let command = krunch.create_command()?;
    krunch.execute_generic_command(command).await?;

    Ok(())
}
