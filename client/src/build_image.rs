use crate::Krunch;
use anyhow::Result;
use log::info;
use std::process::Command;

const DOCKERFILE: &'static str = include_str!("static/Dockerfile");

impl Krunch {
    pub fn execute_host_command(command: &str) -> Result<()> {
        let command = format!(
            "minikube ssh \"mkdir -p krunch-empty && cd krunch-empty && echo '{}' > Dockerfile && docker build -t krunch:latest .\"",
            DOCKERFILE
        );

        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", command.as_str()])
                .output()
                .expect("failed to execute process")
        } else {
            Command::new("sh")
                .arg("-c")
                .arg(command.as_str())
                .output()
                .expect("failed to execute process")
        };

        let blub = String::from_utf8(output.stdout)?;

        info!("{}", blub);

        Ok(())
    }
}
