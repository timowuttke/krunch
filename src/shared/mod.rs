use anyhow::{anyhow, Result};
use crossterm::event::{Event, KeyCode};
use crossterm::{
    cursor::{RestorePosition, SavePosition},
    event, execute,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use kube::config;
use std::io::{stdout, Write};
use std::process::{Command, Output};

pub mod download_urls;
pub mod file_folder_paths;
pub mod windows_registry;

pub const MINIKUBE_HOST: &'static str = "k8s.local";
pub const TLS_SECRET: &'static str = "tls";

pub const KUBECTL_VERSION: &str = "1.23.3";
pub const HELM_VERSION: &str = "3.2.0";
pub const MKCERT_VERSION: &str = "1.4.4";
pub const SKAFFOLD_VERSION: &str = "2.3.1";
pub const K9S_VERSION: &str = "0.27.3";
pub const DOCKER_VERSION: &str = "23.0.4";
pub const BUILDX_VERSION: &str = "0.10.4";

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

pub fn should_continue_as_admin() -> Result<bool> {
    save_term()?;

    print!("Continue as admin (y/N)? ");
    let _ = stdout().flush();

    // Switch to raw mode
    enable_raw_mode()?;

    let input = ' ';

    while !matches!(input, 'y' | 'n' | 'Y' | 'N') {
        if let Event::Key(event) = event::read()? {
            match event.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    if cfg!(target_family = "unix") {
                        let output = Command::new("sudo").arg("-k").output()?;
                        handle_output(output)?;
                    }

                    disable_raw_mode()?;
                    restore_term(0)?;
                    return Ok(true);
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Enter => {
                    disable_raw_mode()?;
                    restore_term(0)?;
                    return Ok(false);
                }
                _ => {}
            }
        }
    }

    Ok(false)
}

pub fn save_term() -> Result<()> {
    let mut stdout = stdout();

    execute!(stdout, SavePosition)?;

    Ok(())
}

pub fn restore_term(lines: u8) -> Result<()> {
    let mut stdout = stdout();

    let lines_up = match lines {
        0 => "".to_string(),
        _ => format!("\x1B[{}A", lines),
    };

    execute!(
        stdout,
        RestorePosition,
        Print(lines_up),
        Clear(ClearType::UntilNewLine),
    )?;
    stdout.flush().unwrap();

    Ok(())
}
