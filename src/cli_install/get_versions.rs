use crate::shared::file_folder_paths::{get_binary_path, get_config_file_path, Binary};
use crate::shared::handle_output;
use anyhow::{anyhow, Result};
use serde::Deserialize;
use serde_json::json;
use std::fs::{create_dir_all, File};
use std::io::{Read, Write};
use std::process::Command;

const KUBECTL_VERSION: &str = "1.23.3";
const HELM_VERSION: &str = "3.2.0";
const MKCERT_VERSION: &str = "1.4.4";
const SKAFFOLD_VERSION: &str = "2.3.1";
const K9S_VERSION: &str = "0.27.3";
const DOCKER_VERSION: &str = "23.0.4";
const BUILDX_VERSION: &str = "0.10.4";

#[derive(Debug, Deserialize)]
pub struct Versions {
    pub kubectl: Option<String>,
    pub helm: Option<String>,
    pub mkcert: Option<String>,
    pub skaffold: Option<String>,
    pub k9s: Option<String>,
    pub docker: Option<String>,
    pub buildx: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct KrunchConfig {
    versions: Versions,
}

pub fn get_expected_versions() -> Result<Versions> {
    let mut file = File::open(get_config_file_path()?)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let config: KrunchConfig = serde_json::from_str(&contents)?;

    Ok(config.versions)
}

pub fn get_actual_versions() -> Result<Versions> {
    Ok(Versions {
        kubectl: get_kubectl_version()?,
        helm: get_helm_version()?,
        mkcert: get_mkcert_version()?,
        skaffold: get_skaffold_version()?,
        k9s: get_k9s_version()?,
        docker: get_docker_version()?,
        buildx: get_buildx_version()?,
    })
}

pub fn create_default_config_if_needed() -> Result<()> {
    if !get_config_file_path()?.exists() {
        let versions = json!({
            "versions": {
                "kubectl": KUBECTL_VERSION,
                "helm": HELM_VERSION,
                "mkcert": MKCERT_VERSION,
                "skaffold": SKAFFOLD_VERSION,
                "k9s": K9S_VERSION,
                "docker": DOCKER_VERSION,
                "buildx": BUILDX_VERSION
            }
        });

        create_dir_all(
            get_config_file_path()?
                .parent()
                .ok_or(anyhow!("failed to create config file path"))?,
        )?;
        let mut file = File::create(get_config_file_path()?)?;
        file.write_all(serde_json::to_string_pretty(&versions)?.as_bytes())?;
    }

    Ok(())
}

fn get_kubectl_version() -> Result<Option<String>> {
    get_any_version(
        Binary::Kubectl,
        "version",
        r#"Client Version:.*GitVersion:"v([\d\.]+)""#,
    )
}

fn get_docker_version() -> Result<Option<String>> {
    get_any_version(Binary::Docker, "version", r"Client:\s+Version:\s+([^\s]+)")
}

fn get_buildx_version() -> Result<Option<String>> {
    get_any_version(Binary::Buildx, "version", r"v(\d+\.\d+\.\d+)")
}

fn get_helm_version() -> Result<Option<String>> {
    get_any_version(Binary::Helm, "version", r#"Version:"v([\d\.]+)""#)
}

fn get_skaffold_version() -> Result<Option<String>> {
    get_any_version(Binary::Skaffold, "version", r"v(\d+\.\d+\.\d+)")
}

fn get_k9s_version() -> Result<Option<String>> {
    get_any_version(Binary::K9S, "version", r"v(\d+\.\d+\.\d+)")
}

fn get_mkcert_version() -> Result<Option<String>> {
    get_any_version(Binary::Mkcert, "-version", r"v(\d+\.\d+\.\d+)")
}

fn get_any_version(binary: Binary, command: &str, regex: &str) -> Result<Option<String>> {
    let output = match Command::new(get_binary_path(binary)?).arg(command).output() {
        Ok(inner) => inner,
        Err(_) => return Ok(None),
    };

    let raw_version_string: String = match handle_output(output) {
        Ok(inner) => inner,
        Err(_) => return Ok(None),
    };

    let re = regex::Regex::new(regex)?;
    let captures = re.captures(raw_version_string.as_str());

    let parsed_output = captures
        .ok_or(anyhow!(
            "not able to detect version: {}",
            raw_version_string
        ))?
        .get(1)
        .ok_or(anyhow!(
            "not able to detect version: {}",
            raw_version_string
        ))?
        .as_str()
        .to_string();
    Ok(Some(parsed_output))
}
