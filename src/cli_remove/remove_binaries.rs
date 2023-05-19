use crate::shared::file_folder_paths::{get_buildx_folder, get_krunch_folder};
use anyhow::Result;
use std::fs;

pub fn remove_binaries() -> Result<()> {
    let mut nothing_done = true;

    let krunch_folder = get_krunch_folder()?;
    if krunch_folder.exists() {
        nothing_done = false;
        fs::remove_dir_all(krunch_folder)?;
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
