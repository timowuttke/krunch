use crate::krunch::Krunch;
use anyhow::Result;
use std::ops::Add;

mod krunch;

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = std::env::args();
    let first = args.nth(1).expect("no pattern given");
    let all_after_first = args.fold(String::new(), |acc, x| acc.add(" ").add(&x));

    let mut krunch = Krunch::new().await?;

    match first.as_str() {
        "init" => krunch.init().await?,
        "get" | "delete" | "describe" => {
            krunch
                .execute_pod_command(
                    "kubectl -n default "
                        .to_string()
                        .add(&*first)
                        .add(&*all_after_first),
                )
                .await?
        }
        "run" => {
            krunch.mount_current_path().await?;
            krunch
                .execute_pod_command("skaffold run -n default".to_string())
                .await?;
            krunch.unmount().await?;
        }
        _ => {}
    }

    Ok(())
}
