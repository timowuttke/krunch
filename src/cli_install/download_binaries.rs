use crate::cli_install::download_urls::get_necessary_downloads;
use crate::cli_install::get_versions::create_default_config_if_needed;
use crate::shared::file_folder_paths::{get_bin_folder, get_buildx_folder};
use anyhow::{anyhow, Result};
use flate2::read::GzDecoder;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Url;
use std::cmp::min;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::{fs, io};
use tar::Archive;
use tempfile::{Builder, TempDir};
use terminal_size::terminal_size;
use walkdir::{DirEntry, WalkDir};

pub async fn download_all() -> Result<()> {
    create_default_config_if_needed()?;
    let downloads = get_necessary_downloads()?;

    if downloads.is_empty() {
        println!("already done")
    } else {
        println!("{:?}", downloads)
    }

    for download in downloads {
        print!("{:<35}", format!("downloading {}", &download.target));
        io::stdout().flush().unwrap();
        download_file(download.source, download.target.as_str()).await?;
        println!("{:<35}success", format!("downloading {}", &download.target));
    }

    Ok(())
}

async fn download_file(url: Url, target_name: &str) -> Result<()> {
    let tmp_dir = Builder::new().tempdir()?;
    let tmp_file_name = url.path_segments().unwrap().last().unwrap();
    let tmp_file_path = tmp_dir.path().join(tmp_file_name);
    let mut tmp_file = File::create(&tmp_file_path)?;

    let response = reqwest::get(url.clone()).await?;
    let total_size = response
        .content_length()
        .ok_or(anyhow!("failed to get content length from '{}'", &url))?;
    let pb = get_progress_bar(total_size, target_name);

    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item?;
        tmp_file.write_all(&chunk)?;
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    handle_tmp_file(tmp_file_path, target_name)?;

    drop(tmp_file);
    tmp_dir.close()?;

    Ok(())
}

fn handle_tmp_file(tmp_file_path: PathBuf, target_name: &str) -> Result<()> {
    let target_path = if target_name.starts_with("docker-buildx") {
        let buildx_folder = get_buildx_folder()?;
        fs::create_dir_all(&buildx_folder)?;
        buildx_folder.join(target_name)
    } else {
        let bin_folder = get_bin_folder()?;
        fs::create_dir_all(&bin_folder)?;
        bin_folder.join(target_name)
    };

    if tmp_file_path.to_str().unwrap().ends_with(".tar.gz")
        || tmp_file_path.to_str().unwrap().ends_with(".tgz")
    {
        let tar_gz = File::open(&tmp_file_path)?;
        let tar = GzDecoder::new(tar_gz);
        let tmp_dir = Builder::new().tempdir()?;
        Archive::new(tar).unpack(&tmp_dir)?;
        find_and_copy_file(tmp_dir, target_name, &target_path)?;
    } else if tmp_file_path.to_str().unwrap().ends_with(".zip") {
        let zip = File::open(&tmp_file_path)?;
        let tmp_dir = Builder::new().tempdir()?;
        zip::ZipArchive::new(zip)?.extract(&tmp_dir)?;
        find_and_copy_file(tmp_dir, target_name, &target_path)?;
    } else {
        fs::copy(tmp_file_path, &target_path)?;
    }

    #[cfg(target_family = "unix")]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(target_path, fs::Permissions::from_mode(0o755))?;
    }

    Ok(())
}

fn find_and_copy_file(dir: TempDir, to_find: &str, target_path: &PathBuf) -> Result<()> {
    for file in WalkDir::new(dir.path())
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e: &DirEntry| e.file_type().is_file())
    {
        if file.file_name() == to_find {
            fs::copy(file.path(), target_path)?;
        }
    }

    Ok(())
}

fn get_progress_bar(total_size: u64, target_name: &str) -> ProgressBar {
    let term_width = terminal_size().unwrap().0 .0 as usize;

    let style = match term_width {
        0..=100 => "\r{msg}{bytes}/{total_bytes}".to_string(),
        _ => "\r{msg}[{bar:30.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec})".to_string(),
    };

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(&style)
            .progress_chars("#>-"),
    );

    pb.set_message(&format!("{:<35}", format!("downloading {}", target_name)));

    pb
}
