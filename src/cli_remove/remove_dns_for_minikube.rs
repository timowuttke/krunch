use crate::shared::file_folder_paths::get_etc_hosts_path;
use crate::shared::{
    handle_output, restore_term, save_term, should_continue_as_admin, MINIKUBE_HOST,
};

use anyhow::{anyhow, Result};
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
        save_term()?;

        if !should_continue_as_admin() {
            restore_term()?;

            return Err(anyhow!("skipped"));
        }
        restore_term()?;

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

        restore_term()?;
        println!("success")
    }

    Ok(())
}

//todo: test this
fn remove_dns_for_minikube_windows() -> Result<()> {
    let etc_hosts_path = get_etc_hosts_path()?;
    let mut data = fs::read_to_string(&etc_hosts_path)?;
    data = data.trim().to_string();

    if data.contains(MINIKUBE_HOST) {
        println!("already done");
    } else {
        if !should_continue_as_admin() {
            return Err(anyhow!("skipped"));
        }

        data = remove_dns_data(data);

        let tmp_file = Builder::new().tempfile()?;
        fs::write(&tmp_file, &data)?;

        let tmp_path = tmp_file.path().to_str().expect("failed to parse tmp path");
        let copy_command = format!("copy /Y \"{}\" \"{}\"", tmp_path, etc_hosts_path.display());

        // Run powershell.exe with Start-Process to trigger a UAC prompt
        let output = Command::new("powershell")
            .arg("Start-Process")
            .arg("cmd.exe")
            .arg("/C")
            .arg(&copy_command)
            .arg("-Verb")
            .arg("RunAs")
            .output()?;
        handle_output(output)?;

        println!("success")
    };

    Ok(())
}

fn remove_dns_data(mut data: String) -> String {
    let re = regex::Regex::new(r"(?m)^.*k8s.local\n").unwrap();
    data = re.replace(&data, "").to_string();

    data
}
