use crate::Krunch;
use anyhow::{anyhow, Result};
use std::fs;
use std::fs::File;
use std::io::{copy, Cursor};
use std::ops::Add;
use std::path::PathBuf;

const KUBECTL_VERSION: &str = "1.26.0";
const KUBECTL_URL: &str = "https://dl.k8s.io/release/vKUBECTL_VERSION/bin/linux/amd64/kubectl";

impl Krunch {
    pub async fn download_all() -> Result<()> {
        Self::download_file_to_bin_folder(
            KUBECTL_URL.replace("KUBECTL_VERSION", KUBECTL_VERSION),
            "kubectl",
        )
        .await?;
        Ok(())
    }

    async fn download_file_to_bin_folder(url: String, fname: &str) -> Result<()> {
        let response = reqwest::get(url).await?;
        let mut content = Cursor::new(response.bytes().await?);

        let bin_folder = Self::get_bin_folder()?;
        fs::create_dir_all(&bin_folder)?;

        let mut dest = File::create(bin_folder.add(fname))?;
        copy(&mut content, &mut dest)?;

        Ok(())
    }

    fn get_bin_folder() -> Result<String> {
        return match home::home_dir() {
            None => return Err(anyhow!("failed to detect home directory")),
            Some(inner) => Ok(format!("{}/.krunch/bin/", inner.display())),
        };
    }
}
