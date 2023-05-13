use crate::shared::file_folder_paths::get_etc_hosts_path;
use crate::shared::{update_etc_hosts, MINIKUBE_HOST};
use anyhow::Result;
use std::fs;

pub fn remove_dns_for_minikube() -> Result<()> {
    let etc_hosts_path = get_etc_hosts_path()?;
    let mut data = fs::read_to_string(&etc_hosts_path)?;
    data = data.trim().to_string();

    if data.contains(MINIKUBE_HOST) {
        data = remove_dns_data(data);
        update_etc_hosts(data)?;

        println!("success");
    } else {
        println!("nothing to do");
    }

    Ok(())
}

fn remove_dns_data(mut data: String) -> String {
    let re = regex::Regex::new(r"(?m)^.*k8s.local\n").unwrap();
    data = re.replace(&data, "").to_string();

    data
}
