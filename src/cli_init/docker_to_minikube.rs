use crate::shared::commands::{get_binary_path, handle_output, Binary};
use anyhow::Result;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::BufReader;
use std::process::Command;

#[cfg(target_family = "unix")]
pub async fn point_docker_to_minikube() -> Result<()> {
    let profile = format!("{}/.profile", home::home_dir().unwrap().display());

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .open(&profile)?;

    let reader = BufReader::new(&file);
    let mut already_exists = false;
    for line in reader.lines() {
        if line?.contains("export DOCKER_HOST") {
            already_exists = true;
            break;
        }
    }

    if already_exists {
        println!("already done");
    } else {
        let (docker_tls_verify, docker_host, docker_cert_path, minikube_active_dockerd) =
            get_docker_env()?;
        writeln!(file, "export DOCKER_TLS_VERIFY=\"{}\"", docker_tls_verify)?;
        writeln!(file, "export DOCKER_HOST=\"{}\"", docker_host)?;
        writeln!(file, "export DOCKER_CERT_PATH=\"{}\"", docker_cert_path)?;
        writeln!(
            file,
            "export MINIKUBE_ACTIVE_DOCKERD=\"{}\"",
            minikube_active_dockerd
        )?;

        println!("success");
    }

    Ok(())
}

#[cfg(target_family = "windows")]
pub async fn point_docker_to_minikube() -> Result<()> {
    use crate::cli_init::create_tls_secret::{handle_output, read_from_environment};

    let current_docker_tls_verify = read_from_environment("DOCKER_TLS_VERIFY");

    if !current_docker_tls_verify.is_err() {
        println!("already done");
    } else {
        let (docker_tls_verify, docker_host, docker_cert_path, minikube_active_dockerd) =
            get_docker_env()?;

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
