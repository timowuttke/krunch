use crate::Krunch;
use anyhow::Result;
use tokio::process::Command;

pub enum Binary {
    Docker,
    Kubectl,
    Helm,
    Skaffold,
    K9S,
    Mkcert,
    Minikube,
}

impl Krunch {
    pub async fn execute_command(binary: Binary, args: &str) -> Result<(String, String, i32)> {
        let extension = if cfg!(target_os = "windows") {
            ".exe"
        } else {
            ""
        };

        let bin = match binary {
            Binary::Docker => format!("{}/docker{}", Self::get_bin_folder()?, extension),
            Binary::Kubectl => format!("{}/kubectl{}", Self::get_bin_folder()?, extension),
            Binary::Helm => format!("{}/helm{}", Self::get_bin_folder()?, extension),
            Binary::Skaffold => format!("{}/skaffold{}", Self::get_bin_folder()?, extension),
            Binary::K9S => format!("{}/k9s{}", Self::get_bin_folder()?, extension),
            Binary::Mkcert => format!("{}/mkcert{}", Self::get_bin_folder()?, extension),
            Binary::Minikube => "minikube".to_string(),
        };

        let command = format!("{} {}", bin, args);

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
            output.status.code().unwrap(),
        ))
    }
}
