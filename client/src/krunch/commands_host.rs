use crate::Krunch;
use anyhow::Result;
use std::env;
use std::process::Stdio;
use tokio::process::Child;
use tokio::process::Command;

impl Krunch {
    async fn execute_host_command(command: &str) -> Result<(String, String)> {
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .arg("-/C")
                .arg(command)
                .output()
                .await
                .expect("failed to execute process")
        } else {
            Command::new("sh")
                .arg("-c")
                .arg(command)
                .output()
                .await
                .expect("failed to execute process")
        };

        Ok((
            String::from_utf8(output.stdout)?,
            String::from_utf8(output.stderr)?,
        ))
    }

    pub fn mount_current_path() -> Result<Child> {
        let current_dir_path_buff = env::current_dir()?;
        let current_dir_str = current_dir_path_buff.as_path().to_str().unwrap();

        let child = Command::new("minikube")
            .arg("mount")
            .arg(format!("{}:/krunch", current_dir_str).as_str())
            .stdout(Stdio::null())
            .spawn()?;

        Ok(child)
    }
}
