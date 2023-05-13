use crate::shared::file_folder_paths::get_etc_hosts_path;
use crate::shared::{copy_as_admin_windows, handle_output, MINIKUBE_HOST};
use anyhow::Result;
use std::fs;
use std::process::Command;
use tempfile::Builder;

pub fn remove_dns_for_minikube() -> Result<()> {
    if cfg!(target_family = "unix") {
        remove_dns_for_minikube_unix()?;
    } else if cfg!(target_family = "windows") {
        remove_dns_for_minikube_windows()?;
    }

    Ok(())
}

fn remove_dns_for_minikube_unix() -> Result<()> {
    let etc_hosts_path = get_etc_hosts_path()?;
    let mut data = fs::read_to_string(&etc_hosts_path)?;
    data = data.trim().to_string();

    if !data.contains(MINIKUBE_HOST) {
        println!("nothing to do");
    } else {
        data = remove_dns_data(data);

        let tmp_file = Builder::new().tempfile()?;
        fs::write(&tmp_file, &data)?;

        let tmp_path = tmp_file.path().to_str().expect("failed to parse tmp path");

        let output = Command::new("sudo")
            .arg("mv")
            .arg(tmp_path)
            .arg(&etc_hosts_path)
            .output()?;
        handle_output(output)?;

        println!("success")
    }

    Ok(())
}

//todo: test this
fn remove_dns_for_minikube_windows() -> Result<()> {
    let etc_hosts_path = get_etc_hosts_path()?;
    let mut data = fs::read_to_string(&etc_hosts_path)?;
    data = data.trim().to_string();

    if !data.contains(MINIKUBE_HOST) {
        println!("nothing to do");
    } else {
        data = remove_dns_data(data);

        let tmp_file = Builder::new().tempfile()?;
        fs::write(&tmp_file, &data)?;

        copy_as_admin_windows(tmp_file.path().to_path_buf(), etc_hosts_path)?;

        println!("success")
    };

    Ok(())
}

fn remove_dns_data(mut data: String) -> String {
    let re = regex::Regex::new(r"(?m)^.*k8s.local\n").unwrap();
    data = re.replace(&data, "").to_string();

    data
}
