use crate::Krunch;
use anyhow::Result;
use log::info;
use std::process::Command;

impl Krunch {
    pub fn execute_host_command(command: &str) -> Result<()> {
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", command])
                .output()
                .expect("failed to execute process")
        } else {
            Command::new("sh")
                .arg("-c")
                .arg(command)
                .output()
                .expect("failed to execute process")
        };

        let blub = String::from_utf8(output.stdout)?;
        let blub3 = String::from_utf8(output.stderr)?;

        info!("{} and {}", blub, blub3);

        Ok(())
    }
}
