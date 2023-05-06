use crate::shared::file_folder_paths::{get_bin_folder, get_buildx_folder};
use anyhow::Result;
use std::fs;

pub fn remove_binaries() -> Result<()> {
    let mut nothing_done = true;

    let bin_folder = get_bin_folder()?;
    if bin_folder.exists() {
        nothing_done = false;
        fs::remove_dir_all(bin_folder)?;
    }

    let extension = if cfg!(target_os = "windows") {
        ".exe"
    } else {
        ""
    };

    let mut buildx_file_path = get_buildx_folder()?;
    let build_file = format!("docker-buildx{}", extension);
    buildx_file_path.push(build_file);

    if buildx_file_path.exists() {
        nothing_done = false;
        fs::remove_file(buildx_file_path)?;
    }

    if nothing_done {
        println!("nothing to do")
    } else {
        println!("success")
    }

    Ok(())
}
