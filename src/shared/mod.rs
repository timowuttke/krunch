use anyhow::{anyhow, Result};
use std::process::Output;

pub mod consts;
pub mod download_urls;
pub mod file_folder_paths;
pub mod windows_registry;

pub fn handle_output(output: Output) -> Result<String> {
    let stdout = String::from_utf8(output.stdout.to_vec())?;
    let stdout = stdout.trim().to_string();

    let stderr = String::from_utf8(output.stderr.to_vec())?;
    let stderr = stderr.trim().to_string();

    if !output.status.success() {
        return if !stderr.is_empty() {
            Err(anyhow!(stderr))
        } else if !stdout.is_empty() {
            Err(anyhow!(stdout))
        } else {
            Err(anyhow!("command failed without output"))
        };
    }

    Ok(stdout)
}

pub async fn get_k8s_client() -> Result<kube::Client> {
    let client = match kube::Client::try_default().await {
        Ok(inner) => inner,
        Err(err) => {
            return Err(anyhow!(
                "failed to load cluster config: {}",
                err.to_string()
            ));
        }
    };

    match client.apiserver_version().await {
        Ok(inner) => inner,
        Err(_) => {
            return Err(anyhow!(
                "failed to connect to cluster, is minikube running?"
            ));
        }
    };

    // ToDo: make sure the context is minikube

    Ok(client)
}
