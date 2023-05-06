use crate::shared::commands::get_bin_folder;
use anyhow::Result;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::BufReader;

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
    use crate::cli_init::create_tls_secret::{read_from_environment, write_to_environment};

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
