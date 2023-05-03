use crate::Krunch;
use anyhow::{anyhow, Result};
use flate2::read::GzDecoder;
use reqwest::Url;
use std::fs::File;
use std::io::prelude::*;
use std::io::{copy, Cursor};
use std::path::{Path, PathBuf};
use std::{fs, io};
use tar::Archive;
use tempfile::{Builder, TempDir};
use walkdir::{DirEntry, WalkDir};

impl Krunch {
    pub async fn download_all() -> Result<()> {
        let downloads = Self::get_downloads();

        for download in downloads {
            print!("downloading {:<18}", &download.target);
            io::stdout().flush().unwrap();

            if download.target.starts_with("docker-buildx")
                && Path::new(&format!(
                    "{}/{}",
                    Self::get_buildx_folder()?,
                    download.target
                ))
                .exists()
            {
                println!("already done")
            } else if Path::new(&format!("{}/{}", Self::get_bin_folder()?, download.target))
                .exists()
            {
                println!("already done")
            } else {
                Self::download_file(download.source, download.target.as_str()).await?;
                println!("success")
            }
        }

        print!("{:30}", "adding tools to PATH");
        io::stdout().flush().unwrap();

        if let Err(_) = Self::add_bin_folder_to_path().await {
            println!("failed, continuing without")
        }

        print!("{:30}", "point docker cli to minikube");
        io::stdout().flush().unwrap();

        if let Err(_) = Self::point_docker_to_minikube().await {
            println!("failed, continuing without")
        }

        Ok(())
    }

    async fn download_file(url: Url, target_name: &str) -> Result<()> {
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
        let target_path;

        if target_name.starts_with("docker-buildx") {
            let buildx_folder = Self::get_buildx_folder()?;
            fs::create_dir_all(&buildx_folder)?;

            target_path = format!("{}/{}", buildx_folder, target_name);
        } else {
            let bin_folder = Self::get_bin_folder()?;
            fs::create_dir_all(&bin_folder)?;

            target_path = format!("{}/{}", bin_folder, target_name);
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

        #[cfg(target_family = "unix")]
        {
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

    pub fn get_bin_folder() -> Result<String> {
        return match home::home_dir() {
            None => return Err(anyhow!("failed to detect home directory")),
            Some(inner) => Ok(format!("{}/.krunch/bin", inner.display())),
        };
    }

    pub fn get_buildx_folder() -> Result<String> {
        return match home::home_dir() {
            None => return Err(anyhow!("failed to detect home directory")),
            Some(inner) => Ok(format!("{}/.docker/cli-plugins", inner.display())),
        };
    }
}
