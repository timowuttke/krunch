use crate::shared::file_folder_paths::get_etc_hosts_path;
use anyhow::{anyhow, Result};
use kube::config;
use std::fs;
use std::fs::File;
use std::io::{stdin, stdout, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::Builder;

pub mod file_folder_paths;
pub mod windows_registry;

#[cfg(windows)]
pub const LINE_ENDING: &str = "\r\n";
#[cfg(not(windows))]
pub const LINE_ENDING: &str = "\n";
pub const MINIKUBE_HOST: &str = "k8s.local";
pub const TLS_SECRET: &str = "tls";

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

pub async fn get_minikube_client() -> Result<kube::Client> {
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
        Ok(_) => (),
        Err(_) => {
            return Err(anyhow!(
                "failed to connect to cluster, is minikube running?"
            ));
        }
    };

    let kubeconfig = config::Kubeconfig::read()?;
    if kubeconfig.current_context != Some("minikube".to_string()) {
        return Err(anyhow!(
            "not connected to minikube, current context is {}",
            kubeconfig.current_context.unwrap()
        ));
    }

    Ok(client)
}

pub fn update_etc_hosts(data: String) -> Result<()> {
    let tmp_file = Builder::new().tempfile()?;
    fs::write(&tmp_file, data)?;

    if cfg!(target_family = "unix") {
        copy_as_admin_unix(&tmp_file.path().to_path_buf(), &get_etc_hosts_path()?)?;
    } else if cfg!(target_family = "windows") {
        copy_as_admin_windows(tmp_file.path(), &get_etc_hosts_path()?)?;
    }

    Ok(())
}

fn copy_as_admin_unix(from: &PathBuf, to: &PathBuf) -> Result<()> {
    let output = Command::new("sudo").arg("-p").arg("[sudo] ").arg("mv").arg(from).arg(to).output()?;

    handle_output(output)?;

    Ok(())
}

fn copy_as_admin_windows(from: &Path, to: &Path) -> Result<()> {
    let tmp_dir = Builder::new().tempdir()?;
    let tmp_file_path = tmp_dir.path().join("krunch");

    let copy_command = format!(
        "'Copy-Item -Path \"{}\" -Destination \"{}\" -Force 2> \"{}\"'",
        from.display(),
        to.display(),
        tmp_file_path.display()
    );

    let result = {
        let output = Command::new("powershell")
            .arg("Start-Process")
            .arg("-FilePath")
            .arg("powershell")
            .arg("-WindowStyle")
            .arg("Hidden")
            .arg("-Wait")
            .arg("-Verb")
            .arg("RunAs")
            .arg("-ArgumentList")
            .arg(&copy_command)
            .output()?;

        handle_output(output)?;

        let mut file = File::open(&tmp_file_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        let contents = String::from_utf8_lossy(&buffer);

        if contents.is_empty() {
            Ok(())
        } else {
            Err(anyhow!("copy failure: {}", contents))
        }
    };

    tmp_dir.close()?;
    result?;

    Ok(())
}

pub fn should_continue_as_admin() -> Result<bool> {
    let mut input = String::new();

    loop {
        print!(
            "Warning: Modifying etc/hosts and the root store requires admin rights. Continue (y/N)? "
        );
        stdout().flush()?;

        input.clear();
        stdin().read_line(&mut input)?;

        match input.trim() {
            "y" | "Y" => {
                if cfg!(target_family = "unix") {
                    let output = Command::new("sudo").arg("-k").output()?;
                    handle_output(output)?;

                    let output = Command::new("sudo").arg("true").output()?;
                    handle_output(output)?;
                }
                return Ok(true);
            }
            "n" | "N" | "" => return Ok(false),
            _ => {
                println!("Invalid response. Please enter 'y', 'Y', 'n', 'N', or press enter.");
                continue;
            }
        }
    }
}
