use crate::krunch::Krunch;
use anyhow::Result;
use std::ops::Add;

mod installer;
mod krunch;

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = std::env::args();
    let first = args.nth(1).expect("no pattern given");
    let all_after_first = args.fold(String::new(), |acc, x| acc.add(" ").add(&x));

    let krunch = Krunch::new().await?;

    match first.as_str() {
        "init" => krunch.init().await?,
        "bomb" => {
            krunch.bomb(all_after_first).await?;
        }
        _ => {}
    }

    Ok(())
}
