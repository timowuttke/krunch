use crate::cli_init::downloads::get_bin_folder;
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

pub fn get_docker_env() -> Result<(String, String, String, String)> {
    let output = Command::new(get_binary_path(Binary::Minikube)?)
        .arg("docker-env")
        .arg("--shell")
        .arg("bash")
        .output()
        .expect("failed to execute process");

    let docker_env = get_stdout_and_handle_errors(output)?;

    Ok(parse_env_string(docker_env.as_str()))
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
    let result = tmp.splitn(2, "C:\\").last().unwrap().trim();

    Ok(result.to_string())
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

pub fn get_stdout_and_handle_errors(output: Output) -> Result<String> {
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

fn parse_env_string(docker_env_bash: &str) -> (String, String, String, String) {
    let mut docker_tls_verify = String::new();
    let mut docker_host = String::new();
    let mut docker_cert_path = String::new();
    let mut minikube_active_dockerd = String::new();

    for line in docker_env_bash.lines() {
        let parts: Vec<&str> = line.split('=').collect();
        if parts.len() == 2 {
            match parts[0] {
                "export DOCKER_TLS_VERIFY" => {
                    docker_tls_verify = parts[1].to_string().replace("\"", "")
                }
                "export DOCKER_HOST" => docker_host = parts[1].to_string().replace("\"", ""),
                "export DOCKER_CERT_PATH" => {
                    docker_cert_path = parts[1].to_string().replace("\"", "")
                }
                "export MINIKUBE_ACTIVE_DOCKERD" => {
                    minikube_active_dockerd = parts[1].to_string().replace("\"", "")
                }
                _ => (),
            }
        }
    }

    (
        docker_tls_verify,
        docker_host,
        docker_cert_path,
        minikube_active_dockerd,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_env_string() {
        let env_string = "export DOCKER_TLS_VERIFY=\"1\"\nexport DOCKER_HOST=\"tcp://192.168.59.101:2376\"\nexport DOCKER_CERT_PATH=\"/home/timo/.minikube/certs\"\nexport MINIKUBE_ACTIVE_DOCKERD=\"minikube\"\n\n# To point your shell to minikube's docker-daemon, run:\n# eval $(minikube -p minikube docker-env)";
        let (docker_tls_verify, docker_host, docker_cert_path, minikube_active_dockerd) =
            parse_env_string(env_string);

        assert_eq!(docker_tls_verify, "1");
        assert_eq!(docker_host, "tcp://192.168.59.101:2376");
        assert_eq!(docker_cert_path, "/home/timo/.minikube/certs");
        assert_eq!(minikube_active_dockerd, "minikube");
    }
}
