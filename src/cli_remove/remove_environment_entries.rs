use crate::shared::file_folder_paths::{get_bin_folder, get_shell_profile_path};
use crate::shared::windows_registry::{
    delete_from_environment, read_from_environment, write_to_environment,
};
use anyhow::{anyhow, Result};
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::{BufRead, BufReader};

pub fn remove_environment_entries() -> Result<()> {
    if cfg!(target_family = "unix") {
        remove_environment_entries_unix()?;
    } else if cfg!(target_family = "windows") {
        remove_environment_entries_windows()?;
    }

    Ok(())
}

fn remove_environment_entries_unix() -> Result<()> {
    let path = get_shell_profile_path()?;

    if !path.exists() {
        return Err(anyhow!("File not found"));
    }

    let file = File::open(&path)?;
    let reader = BufReader::new(file);
    let mut original_lines: Vec<String> = Vec::new();
    let mut modified_lines: Vec<String> = Vec::new();

    for line in reader.lines() {
        let line = line?;
        original_lines.push(line.clone());
        if !line.contains("# krunch")
            && !line.contains("DOCKER_TLS_VERIFY")
            && !line.contains("DOCKER_HOST")
            && !line.contains("DOCKER_CERT_PATH")
            && !line.contains("MINIKUBE_ACTIVE_DOCKERD")
            && !line.contains("/.krunch")
        {
            modified_lines.push(line);
        }
    }

    if original_lines == modified_lines {
        println!("nothing to do");
    } else {
        let mut file = OpenOptions::new().write(true).truncate(true).open(&path)?;
        for line in modified_lines {
            writeln!(file, "{}", line)?;
        }
        println!("success");
    }

    Ok(())
}

fn remove_environment_entries_windows() -> Result<()> {
    let delete_result: Result<_> = (|| {
        delete_from_environment("DOCKER_TLS_VERIFY")?;
        delete_from_environment("DOCKER_HOST")?;
        delete_from_environment("DOCKER_CERT_PATH")?;
        delete_from_environment("MINIKUBE_ACTIVE_DOCKERD")?;

        let current_path = read_from_environment("Path")?;
        let bin_folder = get_bin_folder()?.display().to_string().replace("/", "\\");
        let new_path = current_path.replace(&format!(";{}", bin_folder), "");

        write_to_environment("Path", new_path)?;

        Ok(())
    })();

    if delete_result.is_err() {
        println!("already done");
    } else {
        println!("success");
    }

    Ok(())
}
