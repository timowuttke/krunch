use crate::krunch::Krunch;
use anyhow::Result;
use std::ops::Add;

mod installer;
mod krunch;

#[tokio::main]
async fn main() -> Result<()> {
    Krunch::download_all().await?;

    return Ok(());

    let mut args = std::env::args();
    let first = args.nth(1).expect("no pattern given");
    let all_after_first = args.fold(String::new(), |acc, x| acc.add(" ").add(&x));

    let mut krunch = Krunch::new().await?;

    match first.as_str() {
        "new" => Krunch::download_all().await?,
        "init" => krunch.init().await?,
        "get" | "delete" | "describe" => {
            krunch
                .execute_pod_command(
                    "kubectl -n default "
                        .to_string()
                        .add(&*first)
                        .add(&*all_after_first),
                    true,
                    true,
                )
                .await?;
        }
        "run" => {
            krunch.mount_current_path().await?;
            krunch
                .execute_pod_command("skaffold run -n default".to_string(), true, true)
                .await?;
            krunch.unmount().await?;
        }
        "bomb" => {
            krunch.bomb(all_after_first).await?;
        }
        _ => {}
    }

    Ok(())
}
