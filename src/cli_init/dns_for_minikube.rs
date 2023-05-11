use crate::shared::file_folder_paths::{get_binary_path, get_etc_hosts_path, Binary};
use crate::shared::handle_output;

use anyhow::Result;
use std::fs;
use std::process::Command;
use tempfile::Builder;

pub fn add_dns_for_minikube() -> Result<()> {
    if cfg!(target_family = "unix") {
        add_dns_for_minikube_unix()?;
    } else if cfg!(target_family = "windows") {
        add_dns_for_minikube_windows()?;
    }

    Ok(())
}

fn add_dns_for_minikube_unix() -> Result<()> {
    let etc_hosts_path = get_etc_hosts_path()?;
    let mut data = fs::read_to_string(&etc_hosts_path)?;
    data = data.trim().to_string();

    let minikube_ip = get_minikube_ip()?;

    if data.contains(&minikube_ip) {
        println!("already done");
    } else {
        let message;

        if data.contains("k8s.local") {
            let re = regex::Regex::new(r"(?m)^.*k8s.local\n").unwrap();
            data = re
                .replace(&data, format!("{}\tk8s.local\n", minikube_ip))
                .to_string();
            message = "minikube ip updated";
        } else {
            let re = regex::Regex::new(r"(?m)^\n").unwrap();
            data = re
                .replace(&data, format!("{}\tk8s.local\n\n", minikube_ip))
                .to_string();

            message = "success";
        };

        let tmp_file = Builder::new().tempfile()?;
        fs::write(&tmp_file, &data)?;

        let tmp_path = tmp_file.path().to_str().expect("failed to parse tmp path");

        let output = Command::new("sudo")
            .arg("mv")
            .arg(tmp_path)
            .arg(&etc_hosts_path)
            .output()?;
        handle_output(output)?;

        println!("{}", message);
    };

    Ok(())
}

fn add_dns_for_minikube_windows() -> Result<()> {
    todo!()
}

fn get_minikube_ip() -> Result<String> {
    let output = Command::new(get_binary_path(Binary::Minikube)?)
        .arg("ip")
        .output()
        .expect("failed to execute process");

    let ip = handle_output(output)?;

    Ok(ip)
}
