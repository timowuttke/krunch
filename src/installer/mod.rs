mod urls;

use crate::installer::urls::DownloadUrls;
use crate::Krunch;
use anyhow::{anyhow, Result};
use std::fs;
use std::fs::File;
use std::io::{copy, Cursor};
use std::path::PathBuf;
use tempfile::Builder;

impl Krunch {
    pub async fn download_all() -> Result<()> {
        let dl = DownloadUrls::new();

        Self::download_file_to_bin_folder(dl.kubectl, "kubcetl").await?;

        Ok(())
    }

    async fn download_file_to_bin_folder(url: String, target_name: &str) -> Result<()> {
        let response = reqwest::get(url).await?;

        let tmp_dir = Builder::new().tempdir()?;
        let tmp_file_name = response.url().path_segments().unwrap().last().unwrap();
        let tmp_file_path = tmp_dir.path().join(tmp_file_name);

        let mut content = Cursor::new(response.bytes().await?);
        let mut tmp_file = File::create(&tmp_file_path)?;
        copy(&mut content, &mut tmp_file)?;

        Self::handle_tmp_file(tmp_file_path, target_name)?;

        drop(tmp_file);
        tmp_dir.close()?;

        Ok(())
    }

    fn handle_tmp_file(tmp_file: PathBuf, target_name: &str) -> Result<()> {
        let mut bin_folder = Self::get_bin_folder()?;
        fs::create_dir_all(&bin_folder)?;

        bin_folder.push_str(target_name);
        fs::copy(tmp_file, bin_folder)?;

        Ok(())
    }

    fn get_bin_folder() -> Result<String> {
        return match home::home_dir() {
            None => return Err(anyhow!("failed to detect home directory")),
            Some(inner) => Ok(format!("{}/.krunch/bin/", inner.display())),
        };
    }
}
