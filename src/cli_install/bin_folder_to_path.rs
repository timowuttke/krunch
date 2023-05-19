use crate::shared::file_folder_paths::{get_bin_folder, get_shell_profile_path};
use crate::shared::windows_registry::{read_from_environment, write_to_environment};
use anyhow::Result;
use std::fs;

pub async fn add_bin_folder_to_path() -> Result<()> {
    if cfg!(target_family = "unix") {
        add_bin_folder_to_path_unix()?;
    } else if cfg!(target_family = "windows") {
        add_bin_folder_to_path_windows()?;
    }

    Ok(())
}

fn add_bin_folder_to_path_unix() -> Result<()> {
    let profile_path = get_shell_profile_path()?;
    let bin_folder = get_bin_folder()?;

    let mut data = fs::read_to_string(&profile_path)?;
    data = data.trim().to_string();

    if data.contains(&bin_folder.display().to_string()) {
        println!("already done");
    } else {
        let conditional_path_export = format!(
            "if [[ \":$PATH:\" != *\":{}:\"* ]]; then export PATH=\"{}:$PATH\"; fi",
            bin_folder.display(),
            bin_folder.display()
        );

        data.push_str("\n\n# krunch\n");
        data.push_str(conditional_path_export.as_str());
        data.push_str("\n\n");

        fs::write(profile_path, data)?;
        println!("success");
    };

    Ok(())
}

fn add_bin_folder_to_path_windows() -> Result<()> {
    let current_path = read_from_environment("Path")?;
    let bin_folder = get_bin_folder()?.display().to_string().replace('/', "\\");

    if current_path.contains(&bin_folder) {
        println!("already done");
    } else {
        let divider = if current_path.ends_with(';') { "" } else { ";" };
        let new_path = format!("{}{}{};", current_path, divider, bin_folder);
        write_to_environment("Path", new_path)?;

        println!("success");
    }

    Ok(())
}
