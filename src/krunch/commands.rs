use crate::Krunch;
use anyhow::{anyhow, Result};
use serde_json::Value;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;

pub const MINIKUBE_HOST: &'static str = "k8s.local";

enum Binary {
    _Docker,
    _Kubectl,
    _Helm,
    _Skaffold,
    _K9S,
    Mkcert,
    Minikube,
}

impl Krunch {
    // todo: move below function into an "environment.rs" mod
    pub async fn point_docker_to_minikube() -> Result<()> {
        if cfg!(target_family = "unix") {
            let profile = format!("{}/.profile", home::home_dir().unwrap().display());

            let mut file = OpenOptions::new()
                .read(true)
                .write(true)
                .append(true)
                .open(&profile)?;

            //todo: echo $PATH
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
                let docker_env = Self::get_docker_env()?;
                for line in docker_env.lines() {
                    if line.starts_with("export") {
                        writeln!(file, "{}", line)?;
                    }
                }

                writeln!(file, "# krunch end")?;

                println!("success");
            }
        };

        Ok(())
    }

    pub async fn add_bin_folder_to_path() -> Result<()> {
        if cfg!(target_family = "unix") {
            let profile = format!("{}/.profile", home::home_dir().unwrap().display());

            let mut file = OpenOptions::new()
                .read(true)
                .write(true)
                .append(true)
                .open(&profile)?;

            //todo: echo $PATH
            let reader = BufReader::new(&file);
            let mut already_exists = false;
            for line in reader.lines() {
                if line?.contains("# krunch start") {
                    already_exists = true;
                    break;
                }
            }

            if already_exists {
                println!("already done");
            } else {
                writeln!(file, "\n# krunch start")?;
                writeln!(file, "export PATH=\"$HOME/.krunch/bin:$PATH\"")?;
                println!("success");
            }
        };

        if cfg!(target_os = "windows") {
            let current_path = Self::get_windows_path_variable()?;

            //todo: use PathBuff
            let win_bin_folder = Self::get_bin_folder()?.replace("/", "\\");

            if current_path.contains(&win_bin_folder) {
                println!("already done");
            } else {
                let divider = if current_path.ends_with(";") { "" } else { ";" };
                let new_path = format!("{}{}{};", current_path, divider, win_bin_folder);
                Self::write_to_windows_environment("Path", new_path)?;

                println!("success");
            }
        }

        Ok(())
    }

    pub fn get_docker_env() -> Result<String> {
        let output = Command::new(Self::get_binary_path(Binary::Minikube)?)
            .arg("docker-env")
            .output()
            .expect("failed to execute process");

        let docker_env = Self::get_stdout_and_handle_errors(output)?;

        Ok(docker_env)
    }

    pub fn enable_minikube_ingress_addon() -> Result<()> {
        let output = Command::new(Self::get_binary_path(Binary::Minikube)?)
            .arg("addons")
            .arg("enable")
            .arg("ingress")
            .output()
            .expect("failed to execute process");

        Self::get_stdout_and_handle_errors(output)?;

        Ok(())
    }

    pub fn get_minikbe_addons() -> Result<Value> {
        let output = Command::new(Self::get_binary_path(Binary::Minikube)?)
            .arg("addons")
            .arg("list")
            .arg("--output")
            .arg("json")
            .output()
            .expect("failed to execute process");

        let value: Value = serde_json::from_str(&*Self::get_stdout_and_handle_errors(output)?)?;

        Ok(value)
    }

    pub fn write_to_windows_environment(key: &str, value: String) -> Result<()> {
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

        Self::get_stdout_and_handle_errors(output)?;

        Ok(())
    }

    pub fn get_windows_path_variable() -> Result<String> {
        let output = Command::new("echo")
            .arg("%PATH%")
            .output()
            .expect("failed to execute process");

        let tmp = Self::get_stdout_and_handle_errors(output)?;

        Ok(tmp.trim().to_string())
    }

    pub fn install_local_ca() -> Result<()> {
        let output = Command::new(Self::get_binary_path(Binary::Mkcert)?)
            .arg("--install")
            .output()
            .expect("failed to execute process");

        Self::get_stdout_and_handle_errors(output)?;

        Ok(())
    }

    pub fn create_certificate_files() -> Result<()> {
        let output = Command::new(Self::get_binary_path(Binary::Mkcert)?)
            .arg(MINIKUBE_HOST)
            .output()
            .expect("failed to execute process");

        Self::get_stdout_and_handle_errors(output)?;

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
            Binary::_Docker => {
                let path_str = format!("{}/docker{}", Self::get_bin_folder()?, extension);
                PathBuf::from(path_str)
            }
            Binary::_Kubectl => {
                let path_str = format!("{}/kubectl{}", Self::get_bin_folder()?, extension);
                PathBuf::from(path_str)
            }
            Binary::_Helm => {
                let path_str = format!("{}/helm{}", Self::get_bin_folder()?, extension);
                PathBuf::from(path_str)
            }
            Binary::_Skaffold => {
                let path_str = format!("{}/skaffold{}", Self::get_bin_folder()?, extension);
                PathBuf::from(path_str)
            }
            Binary::_K9S => {
                let path_str = format!("{}/k9s{}", Self::get_bin_folder()?, extension);
                PathBuf::from(path_str)
            }
            Binary::Mkcert => {
                let path_str = format!("{}/mkcert{}", Self::get_bin_folder()?, extension);
                PathBuf::from(path_str)
            }
            Binary::Minikube => PathBuf::from("minikube"),
        };

        Ok(path)
    }
}
