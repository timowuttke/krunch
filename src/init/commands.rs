use crate::init::downloads::get_bin_folder;
use crate::r#const::MINIKUBE_HOST;
use anyhow::{anyhow, Result};
use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;

enum Binary {
    _Docker,
    _Kubectl,
    _Helm,
    _Skaffold,
    _K9S,
    Mkcert,
    Minikube,
}

pub fn get_docker_env() -> Result<String> {
    let output = Command::new(get_binary_path(Binary::Minikube)?)
        .arg("docker-env")
        .output()
        .expect("failed to execute process");

    let docker_env = get_stdout_and_handle_errors(output)?;

    Ok(docker_env)
}

pub fn enable_minikube_ingress_addon() -> Result<()> {
    let output = Command::new(get_binary_path(Binary::Minikube)?)
        .arg("addons")
        .arg("enable")
        .arg("ingress")
        .output()
        .expect("failed to execute process");

    get_stdout_and_handle_errors(output)?;

    Ok(())
}

pub fn get_minikbe_addons() -> Result<Value> {
    let output = Command::new(get_binary_path(Binary::Minikube)?)
        .arg("addons")
        .arg("list")
        .arg("--output")
        .arg("json")
        .output()
        .expect("failed to execute process");

    let value: Value = serde_json::from_str(&*get_stdout_and_handle_errors(output)?)?;

    Ok(value)
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

    get_stdout_and_handle_errors(output)?;

    let output = Command::new("SETX")
        .arg("USERNAME")
        .arg("%USERNAME%")
        .output()
        .expect("failed to execute process");

    get_stdout_and_handle_errors(output)?;

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

    let tmp = get_stdout_and_handle_errors(output)?;

    Ok(tmp.trim().to_string())
}

pub fn install_local_ca() -> Result<()> {
    let output = Command::new(get_binary_path(Binary::Mkcert)?)
        .arg("--install")
        .output()
        .expect("failed to execute process");

    get_stdout_and_handle_errors(output)?;

    Ok(())
}

pub fn create_certificate_files() -> Result<()> {
    let output = Command::new(get_binary_path(Binary::Mkcert)?)
        .arg(MINIKUBE_HOST)
        .output()
        .expect("failed to execute process");

    get_stdout_and_handle_errors(output)?;

    Ok(())
}

fn get_stdout_and_handle_errors(output: Output) -> Result<String> {
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

fn get_binary_path(binary: Binary) -> Result<PathBuf> {
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
