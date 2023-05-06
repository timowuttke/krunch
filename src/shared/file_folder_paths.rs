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
        Some(inner) => Ok(inner.join(".krunch")),
    };
}

pub fn get_buildx_folder() -> Result<PathBuf> {
    return match home::home_dir() {
        None => return Err(anyhow!("failed to detect home directory")),
        Some(inner) => Ok(inner.join(".docker/cli-plugins")),
    };
}

pub fn get_shell_profile_path() -> Result<PathBuf> {
    let mut profile_path = home::home_dir().expect("no home directory found");

    // todo: check for different shell variants, e.g. fn get_unix_file_for_path and make sure file exists
    profile_path.push(".profile");

    Ok(profile_path)
}
