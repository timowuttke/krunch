mod urls;

use crate::installer::urls::DownloadUrls;
use crate::Krunch;
use anyhow::{anyhow, Result};
use flate2::read::GzDecoder;
use reqwest::Url;
use std::fs;
use std::fs::File;
use std::io::{copy, Cursor};
use std::path::PathBuf;
use tar::Archive;
use tempfile::{Builder, TempDir};
use walkdir::{DirEntry, WalkDir};

impl Krunch {
    pub async fn download_all() -> Result<()> {
        let url = DownloadUrls::new()?;

        // Self::download_file_to_bin_folder(url.docker, "docker").await?;
        // Self::download_file_to_bin_folder(url.kubectl, "kubectl").await?;
        // Self::download_file_to_bin_folder(url.helm, "helm").await?;
        Self::download_file_to_bin_folder(url.mkcert, "mkcert").await?;
        // Self::download_file_to_bin_folder(url.skaffold, "skaffold").await?;
        // Self::download_file_to_bin_folder(url.k9s, "k9s").await?;

        Ok(())
    }

    async fn download_file_to_bin_folder(url: Url, target_name: &str) -> Result<()> {
        let tmp_dir = Builder::new().tempdir()?;
        let tmp_file_name = url.path_segments().unwrap().last().unwrap();
        let tmp_file_path = tmp_dir.path().join(tmp_file_name);
        let mut tmp_file = File::create(&tmp_file_path)?;

        let response = reqwest::get(url).await?;
        let mut content = Cursor::new(response.bytes().await?);
        copy(&mut content, &mut tmp_file)?;

        Self::handle_tmp_file(tmp_file_path, target_name)?;

        drop(tmp_file);
        tmp_dir.close()?;

        Ok(())
    }

    fn handle_tmp_file(tmp_file_path: PathBuf, target_name: &str) -> Result<()> {
        let bin_folder = Self::get_bin_folder()?;
        fs::create_dir_all(&bin_folder)?;

        let mut target_path = format!("{}/{}", bin_folder, target_name);

        if cfg!(target_os = "windows") {
            target_path.push_str(".exe");
        }

        if tmp_file_path.to_str().unwrap().ends_with(".tar.gz")
            || tmp_file_path.to_str().unwrap().ends_with(".tgz")
        {
            let tar_gz = File::open(&tmp_file_path)?;
            let tar = GzDecoder::new(tar_gz);
            let tmp_dir = Builder::new().tempdir()?;
            Archive::new(tar).unpack(&tmp_dir)?;
            Self::find_and_copy_file(tmp_dir, target_name, &target_path)?;
        } else if tmp_file_path.to_str().unwrap().ends_with(".zip") {
            let zip = File::open(&tmp_file_path)?;
            let tmp_dir = Builder::new().tempdir()?;
            zip::ZipArchive::new(zip)?.extract(&tmp_dir)?;
            Self::find_and_copy_file(tmp_dir, target_name, &target_path)?;
        } else {
            fs::copy(tmp_file_path, &target_path)?;
        }

        if cfg!(target_familiy = "unix") {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(target_path, fs::Permissions::from_mode(0o755))?;
        }

        Ok(())
    }

    fn find_and_copy_file(
        dir: TempDir,
        to_find: &str,
        target_path: &String,
    ) -> std::io::Result<()> {
        for file in WalkDir::new(dir.path())
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e: &DirEntry| e.file_type().is_file())
        {
            if file.file_name() == to_find {
                fs::copy(file.path(), &target_path)?;
            }
        }

        Ok(())
    }

    fn get_bin_folder() -> Result<String> {
        return match home::home_dir() {
            None => return Err(anyhow!("failed to detect home directory")),
            Some(inner) => Ok(format!("{}/.krunch/bin", inner.display())),
        };
    }
}