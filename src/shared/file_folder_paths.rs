use anyhow::{anyhow, Result};
use std::path::PathBuf;

pub enum Binary {
    _Docker,
    _Kubectl,
    _Helm,
    _Skaffold,
    _K9S,
    Mkcert,
    Minikube,
}

pub fn get_binary_path(binary: Binary) -> Result<PathBuf> {
    let extension = if cfg!(target_os = "windows") {
        ".exe"
    } else {
        ""
    };

    let path = match binary {
        Binary::_Docker => get_bin_folder()?.join(format!("docker{}", extension)),
        Binary::_Kubectl => get_bin_folder()?.join(format!("kubectl{}", extension)),
        Binary::_Helm => get_bin_folder()?.join(format!("helm{}", extension)),
        Binary::_Skaffold => get_bin_folder()?.join(format!("skaffold{}", extension)),
        Binary::_K9S => get_bin_folder()?.join(format!("k9s{}", extension)),
        Binary::Mkcert => get_bin_folder()?.join(format!("mkcert{}", extension)),
        Binary::Minikube => PathBuf::from("minikube"),
    };

    Ok(path)
}

pub fn get_bin_folder() -> Result<PathBuf> {
    return match home::home_dir() {
        None => return Err(anyhow!("failed to detect home directory")),
        Some(inner) => Ok(inner.join(".krunch/bin")),
    };
}

pub fn get_buildx_folder() -> Result<PathBuf> {
    return match home::home_dir() {
        None => return Err(anyhow!("failed to detect home directory")),
        Some(inner) => Ok(inner.join(".docker/cli-plugins")),
    };
}
