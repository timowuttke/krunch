use crate::shared::file_folder_paths::{get_binary_path, get_shell_profile_path, Binary};
use crate::shared::handle_output;
use crate::shared::windows_registry::read_from_environment;
use anyhow::Result;
use std::fs;
use std::process::Command;

pub async fn point_docker_to_minikube() -> Result<()> {
    if cfg!(target_family = "unix") {
        point_docker_to_minikube_unix()?;
    } else if cfg!(target_family = "windows") {
        point_docker_to_minikube_windows()?;
    }

    Ok(())
}

fn point_docker_to_minikube_unix() -> Result<()> {
    let (docker_tls_verify, docker_host, docker_cert_path, minikube_active_dockerd) =
        get_docker_env()?;

    let profile_path = get_shell_profile_path()?;

    let mut data = fs::read_to_string(&profile_path)?;
    data = data.trim().to_string();

    if data.contains(&docker_host) {
        println!("already done");
    } else if data.contains("export DOCKER_HOST") {
        let re = regex::Regex::new(r"(?m)^export DOCKER_HOST.*\n").unwrap();
        data = re
            .replace(&data, format!("export DOCKER_HOST=\"{}\"\n", docker_host))
            .to_string();
        fs::write(profile_path, data)?;

        println!("minikube IP updated");
    } else {
        data.push_str(format!("\nexport DOCKER_TLS_VERIFY=\"{}\"", docker_tls_verify).as_str());
        data.push_str(format!("\nexport DOCKER_HOST=\"{}\"", docker_host).as_str());
        data.push_str(format!("\nexport DOCKER_CERT_PATH=\"{}\"", docker_cert_path).as_str());
        data.push_str(
            format!(
                "\nexport MINIKUBE_ACTIVE_DOCKERD=\"{}\"",
                minikube_active_dockerd
            )
            .as_str(),
        );
        data.push_str("\n\n");

        fs::write(profile_path, data)?;

        println!("success");
    }

    Ok(())
}

fn point_docker_to_minikube_windows() -> Result<()> {
    let (docker_tls_verify, docker_host, docker_cert_path, minikube_active_dockerd) =
        get_docker_env()?;

    let current_docker_host = read_from_environment("DOCKER_HOST");

    if let Ok(current_docker_host) = current_docker_host {
        if docker_host == current_docker_host {
            println!("already done");
        } else {
            let output = Command::new("SETX")
                .arg("DOCKER_HOST")
                .arg(docker_host)
                .output()
                .expect("failed to execute process");

            handle_output(output)?;

            println!("minikube IP updated");
        }
    } else {
        let output = Command::new("SETX")
            .arg("DOCKER_TLS_VERIFY")
            .arg(docker_tls_verify)
            .output()
            .expect("failed to execute process");

        handle_output(output)?;

        let output = Command::new("SETX")
            .arg("DOCKER_HOST")
            .arg(docker_host)
            .output()
            .expect("failed to execute process");

        handle_output(output)?;

        let output = Command::new("SETX")
            .arg("DOCKER_CERT_PATH")
            .arg(docker_cert_path)
            .output()
            .expect("failed to execute process");

        handle_output(output)?;

        let output = Command::new("SETX")
            .arg("MINIKUBE_ACTIVE_DOCKERD")
            .arg(minikube_active_dockerd)
            .output()
            .expect("failed to execute process");

        handle_output(output)?;

        println!("success");
    }

    Ok(())
}

fn get_docker_env() -> Result<(String, String, String, String)> {
    let output = Command::new(get_binary_path(Binary::Minikube)?)
        .arg("docker-env")
        .arg("--shell")
        .arg("bash")
        .output()
        .expect("failed to execute process");

    let docker_env = handle_output(output)?;

    Ok(parse_env_string(docker_env.as_str()))
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
