use anyhow::{anyhow, Result};
use std::env;
use std::path::PathBuf;

pub enum Binary {
    Docker,
    Buildx,
    Kubectl,
    Helm,
    Skaffold,
    K9S,
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
        Binary::Docker => get_bin_folder()?.join(format!("docker{}", extension)),
        Binary::Buildx => get_buildx_folder()?.join(format!("docker-buildx{}", extension)),
        Binary::Kubectl => get_bin_folder()?.join(format!("kubectl{}", extension)),
        Binary::Helm => get_bin_folder()?.join(format!("helm{}", extension)),
        Binary::Skaffold => get_bin_folder()?.join(format!("skaffold{}", extension)),
        Binary::K9S => get_bin_folder()?.join(format!("k9s{}", extension)),
        Binary::Mkcert => get_bin_folder()?.join(format!("mkcert{}", extension)),
        Binary::Minikube => PathBuf::from("minikube"),
    };

    Ok(path)
}

pub fn get_krunch_folder() -> Result<PathBuf> {
    let home_dir = home::home_dir().ok_or(anyhow!("failed to detect home directory"))?;
    Ok(home_dir.join(".krunch"))
}

pub fn get_bin_folder() -> Result<PathBuf> {
    let home_dir = home::home_dir().ok_or(anyhow!("failed to detect home directory"))?;
    Ok(home_dir.join(".krunch/bin"))
}

pub fn get_config_file_path() -> Result<PathBuf> {
    let home_dir = home::home_dir().ok_or(anyhow!("failed to detect home directory"))?;
    Ok(home_dir.join(".krunch/config.json"))
}

pub fn get_buildx_folder() -> Result<PathBuf> {
    let home_dir = home::home_dir().ok_or(anyhow!("failed to detect home directory"))?;
    Ok(home_dir.join(".docker/cli-plugins"))
}

pub fn get_shell_profile_path() -> Result<PathBuf> {
    let shell =
        env::var("SHELL").map_err(|_| anyhow!("Failed to get SHELL environment variable"))?;
    if !shell.contains("bash") && !shell.contains("zsh") {
        return Err(anyhow!(
            "Unsupported shell. Only bash and zsh are supported."
        ));
    }

    let home_dir = home::home_dir().ok_or(anyhow!("failed to detect home directory"))?;

    let bash_profiles = vec![".bashrc", ".bash_profile", ".bash_login", ".profile"];
    let zsh_profiles = vec![".zshrc", ".zprofile", ".zlogin"];
    let profiles = if shell.contains("bash") {
        bash_profiles
    } else {
        zsh_profiles
    };

    for profile in profiles {
        let profile_path = home_dir.join(profile);
        if profile_path.exists() {
            return Ok(profile_path);
        }
    }

    Err(anyhow!("No suitable profile file found."))
}

pub fn get_etc_hosts_path() -> Result<PathBuf> {
    if cfg!(target_os = "windows") {
        Ok(PathBuf::from("C:/Windows/System32/Drivers/etc/hosts"))
    } else {
        Ok(PathBuf::from("/etc/hosts"))
    }
}
