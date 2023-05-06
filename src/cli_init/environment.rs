use crate::cli_init::commands::get_docker_env;
use crate::cli_init::downloads::get_bin_folder;
use anyhow::Result;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::BufReader;
use std::process::Command;

#[cfg(target_family = "unix")]
pub async fn add_bin_folder_to_path() -> Result<()> {
    let mut profile_path = home::home_dir().unwrap();
    // todo: check for different shell variants, e.g. fn get_unix_file_for_path and make sure file exists
    profile_path.push(".profile");

    let mut profile = OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .open(&profile_path)?;

    let reader = BufReader::new(&profile);
    let mut already_exists = false;
    let bin_folder = get_bin_folder()?;
    for line in reader.lines() {
        let line = line?;
        if line.contains(&bin_folder.display().to_string()) {
            already_exists = true;
            break;
        }
    }

    if already_exists {
        println!("already done");
    } else {
        writeln!(profile, "\n# krunch")?;
        writeln!(profile, "export PATH=\"{}:$PATH\"", bin_folder.display())?;
        println!("success");
    }

    Ok(())
}

#[cfg(target_family = "windows")]
pub async fn add_bin_folder_to_path() -> Result<()> {
    use crate::cli_init::commands::{read_from_environment, write_to_environment};

    let current_path = read_from_environment("Path")?;
    let bin_folder = get_bin_folder()?.display().to_string().replace("/", "\\");

    if current_path.contains(&bin_folder) {
        println!("already done");
    } else {
        let divider = if current_path.ends_with(";") { "" } else { ";" };
        let new_path = format!("{}{}{};", current_path, divider, bin_folder);
        write_to_environment("Path", new_path)?;

        println!("success");
    }

    Ok(())
}

#[cfg(target_family = "unix")]
pub async fn point_docker_to_minikube() -> Result<()> {
    let profile = format!("{}/.profile", home::home_dir().unwrap().display());

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .open(&profile)?;

    let reader = BufReader::new(&file);
    let mut already_exists = false;
    for line in reader.lines() {
        if line?.contains("export DOCKER_HOST") {
            already_exists = true;
            break;
        }
    }

    if already_exists {
        println!("already done");
    } else {
        let (docker_tls_verify, docker_host, docker_cert_path, minikube_active_dockerd) =
            get_docker_env()?;
        writeln!(file, "export DOCKER_TLS_VERIFY=\"{}\"", docker_tls_verify)?;
        writeln!(file, "export DOCKER_HOST=\"{}\"", docker_host)?;
        writeln!(file, "export DOCKER_CERT_PATH=\"{}\"", docker_cert_path)?;
        writeln!(
            file,
            "export MINIKUBE_ACTIVE_DOCKERD=\"{}\"",
            minikube_active_dockerd
        )?;

        println!("success");
    }

    Ok(())
}

#[cfg(target_family = "windows")]
pub async fn point_docker_to_minikube() -> Result<()> {
    use crate::cli_init::commands::{get_stdout_and_handle_errors, read_from_environment};

    let current_docker_tls_verify = read_from_environment("DOCKER_TLS_VERIFY");

    if !current_docker_tls_verify.is_err() {
        println!("already done");
    } else {
        let (docker_tls_verify, docker_host, docker_cert_path, minikube_active_dockerd) =
            get_docker_env()?;

        let output = Command::new("SETX")
            .arg("DOCKER_TLS_VERIFY")
            .arg(docker_tls_verify)
            .output()
            .expect("failed to execute process");

        get_stdout_and_handle_errors(output)?;

        let output = Command::new("SETX")
            .arg("DOCKER_HOST")
            .arg(docker_host)
            .output()
            .expect("failed to execute process");

        get_stdout_and_handle_errors(output)?;

        let output = Command::new("SETX")
            .arg("DOCKER_CERT_PATH")
            .arg(docker_cert_path)
            .output()
            .expect("failed to execute process");

        get_stdout_and_handle_errors(output)?;

        let output = Command::new("SETX")
            .arg("MINIKUBE_ACTIVE_DOCKERD")
            .arg(minikube_active_dockerd)
            .output()
            .expect("failed to execute process");

        get_stdout_and_handle_errors(output)?;

        println!("success");
    }

    Ok(())
}
