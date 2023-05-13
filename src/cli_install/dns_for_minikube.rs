use crate::shared::file_folder_paths::{get_binary_path, get_etc_hosts_path, Binary};
use crate::shared::{handle_output, update_etc_hosts};
use anyhow::Result;
use std::fs;
use std::process::Command;

pub fn add_dns_for_minikube() -> Result<()> {
    let etc_hosts_path = get_etc_hosts_path()?;
    let mut data = fs::read_to_string(&etc_hosts_path)?;
    data = data.trim().to_string();

    let minikube_ip = get_minikube_ip()?;

    if data.contains(&minikube_ip) {
        println!("already done");
    } else if data.contains("k8s.local") {
        let data = update_dns_data(data, minikube_ip);
        update_etc_hosts(data)?;

        println!("minikube ip updated");
    } else {
        let data = add_dns_data(data, minikube_ip);
        update_etc_hosts(data)?;

        println!("success");
    }

    Ok(())
}

fn add_dns_data(mut data: String, minikube_ip: String) -> String {
    data.push_str("\n\n");

    let re = regex::Regex::new(r"(?m)^\n").unwrap();
    data = re
        .replace(&data, format!("{}\tk8s.local\n\n", minikube_ip))
        .to_string();

    data.trim().to_string()
}

fn update_dns_data(mut data: String, minikube_ip: String) -> String {
    data.push_str("\n\n");

    let re = regex::Regex::new(r"(?m)^.*k8s.local\n").unwrap();
    data = re
        .replace(&data, format!("{}\tk8s.local\n", minikube_ip))
        .to_string();

    data.trim().to_string()
}

fn get_minikube_ip() -> Result<String> {
    let output = Command::new(get_binary_path(Binary::Minikube)?)
        .arg("ip")
        .output()
        .expect("failed to execute process");

    let ip = handle_output(output)?;

    Ok(ip)
}
