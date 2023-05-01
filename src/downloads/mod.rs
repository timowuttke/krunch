mod urls;

use crate::krunch::command::Binary;
use crate::Krunch;
use anyhow::{anyhow, Result};
use flate2::read::GzDecoder;
use reqwest::Url;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::{copy, BufReader, Cursor};
use std::path::{Path, PathBuf};
use std::{fs, io};
use tar::Archive;
use tempfile::{Builder, TempDir};
use tokio::process::Command;
use walkdir::{DirEntry, WalkDir};

impl Krunch {
    pub async fn download_all() -> Result<()> {
        let downloads = Self::get_downloads();

        for download in downloads {
            print!("downloading {:<18}", &download.target);
            io::stdout().flush().unwrap();

            if Path::new(&format!("{}/{}", Self::get_bin_folder()?, download.target)).exists() {
                println!("already exists")
            } else {
                Self::download_file_to_bin_folder(download.source, download.target.as_str())
                    .await?;
                println!("done")
            }
        }

        print!("{:30}", "adding tools to PATH");
        io::stdout().flush().unwrap();

        if let Err(_) = Self::add_bin_folder_to_path().await {
            println!("failed, continuing without")
        }

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

        let target_path = format!("{}/{}", bin_folder, target_name);

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

    pub async fn add_bin_folder_to_path() -> Result<()> {
        if cfg!(target_family = "unix") {
            let profile = format!("{}/.profile", home::home_dir().unwrap().display());

            let mut file = OpenOptions::new()
                .read(true)
                .write(true)
                .append(true)
                .open(&profile)?;

            let reader = BufReader::new(&file);
            let mut already_exists = false;
            for line in reader.lines() {
                if line?.contains("#krunch") {
                    already_exists = true;
                    break;
                }
            }

            if !already_exists {
                writeln!(file, "\n#krunch\nexport PATH=\"$HOME/.krunch/bin:$PATH\"",)?;
            }
        };

        if cfg!(target_os = "windows") {
            let tmp = Self::execute_command(Binary::None, "echo %PATH%").await?.0;
            let current_path = tmp.trim();

            //todo: use PathBuff
            let win_bin_folder = Self::get_bin_folder()?.replace("/", "\\");

            if !current_path.contains(&win_bin_folder) {
                let divider = if current_path.ends_with(";") { "" } else { ";" };

                let new_path = format!("{}{}{};", current_path, divider, win_bin_folder);

                //todo: rething command architecture
                let write_reg_result = Command::new("reg")
                    .arg("add")
                    .arg("HKEY_CURRENT_USER\\Environment")
                    .arg("/v")
                    .arg("Path")
                    .arg("/t")
                    .arg("REG_SZ")
                    .arg("/d")
                    .arg(format!("{}", new_path))
                    .arg("/f")
                    .output()
                    .await
                    .expect("failed to execute process");

                let update_env_result =
                    Self::execute_command(Binary::None, "SETX USERNAME %USERNAME%").await?;

                if !write_reg_result.status.success() || update_env_result.2 != 0 {
                    return Err(anyhow!(
                        "failed to add bin folder to PATH: {}",
                        String::from_utf8(write_reg_result.stderr)?
                    ));
                }
            }
        }

        Ok(())
    }
}
