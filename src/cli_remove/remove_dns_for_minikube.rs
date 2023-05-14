use crate::shared::file_folder_paths::get_etc_hosts_path;
use crate::shared::{update_etc_hosts, LINE_ENDING, MINIKUBE_HOST};
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
    data.push_str(LINE_ENDING);
    data.push_str(LINE_ENDING);

    let re = regex::Regex::new(&format!(r"(?m)^.*k8s.local{}", LINE_ENDING)).unwrap();
    data = re.replace(&data, "").to_string();

    data.trim().to_string()
}
