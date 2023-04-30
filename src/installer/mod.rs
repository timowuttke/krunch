mod urls;

use crate::installer::urls::DownloadUrls;
use crate::Krunch;
use anyhow::{anyhow, Result};
use std::fs;
use std::fs::File;
use std::io::{copy, Cursor};
use std::ops::Add;
use std::path::PathBuf;
use tempfile::Builder;

impl Krunch {
    pub async fn download_all() -> Result<()> {
        let dl = DownloadUrls::new();

        Self::download_file_to_tmp_folder(dl.kubectl).await?;

        Ok(())
    }

    async fn download_file_to_tmp_folder(url: String) -> Result<()> {
        let response = reqwest::get(url).await?;

        let tmp_dir = Builder::new().tempdir()?;
        let file_name = response.url().path_segments().unwrap().last().unwrap();
        let file_path = tmp_dir.path().join(file_name);

        println!("downloaded to {}", &file_path.to_str().unwrap());
        let mut file = File::create(file_path)?;

        // let bin_folder = Self::get_bin_folder()?;
        // fs::create_dir_all(&bin_folder)?;

        let mut content = Cursor::new(response.bytes().await?);
        copy(&mut content, &mut file)?;

        Ok(())
    }

    fn get_bin_folder() -> Result<String> {
        return match home::home_dir() {
            None => return Err(anyhow!("failed to detect home directory")),
            Some(inner) => Ok(format!("{}/.krunch/bin/", inner.display())),
        };
    }
}
