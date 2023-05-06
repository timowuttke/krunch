use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::process::Output;

pub enum Binary {
    _Docker,
    _Kubectl,
    _Helm,
    _Skaffold,
    _K9S,
    Mkcert,
    Minikube,
}

pub fn handle_output(output: Output) -> Result<String> {
    let stdout = String::from_utf8(output.stdout.to_vec())?;
    let stdout = stdout.trim().to_string();

    let stderr = String::from_utf8(output.stderr.to_vec())?;
    let stderr = stderr.trim().to_string();

    if !output.status.success() {
        return if !stderr.is_empty() {
            Err(anyhow!(stderr))
        } else if !stdout.is_empty() {
            Err(anyhow!(stdout))
        } else {
            Err(anyhow!("command failed without output"))
        };
    }

    Ok(stdout)
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

#[cfg(target_family = "windows")]
pub fn write_to_environment(key: &str, value: String) -> Result<()> {
    let output = Command::new("reg")
        .arg("add")
        .arg("HKEY_CURRENT_USER\\Environment")
        .arg("/v")
        .arg(key)
        .arg("/t")
        .arg("REG_SZ")
        .arg("/d")
        .arg(value)
        .arg("/f")
        .output()
        .expect("failed to execute process");

    handle_output(output)?;

    let output = Command::new("SETX")
        .arg("USERNAME")
        .arg("%USERNAME%")
        .output()
        .expect("failed to execute process");

    handle_output(output)?;

    Ok(())
}

#[cfg(target_family = "windows")]
pub fn read_from_environment(key: &str) -> Result<String> {
    let output = Command::new("reg")
        .arg("query")
        .arg("HKEY_CURRENT_USER\\Environment")
        .arg("/v")
        .arg(key)
        .output()
        .expect("failed to execute process");

    let tmp = handle_output(output)?;
    let result = tmp.splitn(2, "C:\\").last().unwrap().trim();

    Ok(result.to_string())
}

pub fn get_bin_folder() -> Result<PathBuf> {
    return match home::home_dir() {
        None => return Err(anyhow!("failed to detect home directory")),
        Some(inner) => Ok(inner.join(".krunch/bin")),
    };
}
