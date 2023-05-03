use crate::Krunch;
use anyhow::{anyhow, Result};
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use tokio::process::Command;

pub enum Binary {
    _Docker,
    _Kubectl,
    _Helm,
    _Skaffold,
    _K9S,
    Mkcert,
    Minikube,
    None,
}

impl Krunch {
    //todo: error handling in commandos, remember minikube
    pub async fn execute_command(binary: Binary, args: &str) -> Result<(String, String, i32)> {
        let extension = if cfg!(target_os = "windows") {
            ".exe"
        } else {
            ""
        };

        let bin = match binary {
            Binary::_Docker => format!("{}/docker{}", Self::get_bin_folder()?, extension),
            Binary::_Kubectl => format!("{}/kubectl{}", Self::get_bin_folder()?, extension),
            Binary::_Helm => format!("{}/helm{}", Self::get_bin_folder()?, extension),
            Binary::_Skaffold => format!("{}/skaffold{}", Self::get_bin_folder()?, extension),
            Binary::_K9S => format!("{}/k9s{}", Self::get_bin_folder()?, extension),
            Binary::Mkcert => format!("{}/mkcert{}", Self::get_bin_folder()?, extension),
            Binary::Minikube => "minikube".to_string(),
            Binary::None => "".to_string(),
        };

        let command = format!("{} {}", bin, args);

        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .arg("-/C")
                .arg(command)
                .output()
                .await
                .expect("failed to execute process")
        } else {
            Command::new("sh")
                .arg("-c")
                .arg(command)
                .output()
                .await
                .expect("failed to execute process")
        };

        Ok((
            String::from_utf8(output.stdout)?,
            String::from_utf8(output.stderr)?,
            output.status.code().unwrap(),
        ))
    }

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
                if line?.contains("# export DOCKER_HOST") {
                    already_exists = true;
                    break;
                }
            }

            if already_exists {
                println!("already done");
            } else {
                let docker_env = Self::execute_command(Binary::None, "minikube docker-env")
                    .await?
                    .0;

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
            let tmp = Self::execute_command(Binary::None, "echo %PATH%").await?.0;
            let current_path = tmp.trim();

            //todo: use PathBuff
            let win_bin_folder = Self::get_bin_folder()?.replace("/", "\\");

            if current_path.contains(&win_bin_folder) {
                println!("already done");
            } else {
                let divider = if current_path.ends_with(";") { "" } else { ";" };

                let new_path = format!("{}{}{};", current_path, divider, win_bin_folder);

                //todo: rethink command architecture
                let write_reg_result = Command::new("reg")
                    .arg("add")
                    .arg("HKEY_CURRENT_USER\\Environment")
                    .arg("/v")
                    .arg("Path")
                    .arg("/t")
                    .arg("REG_SZ")
                    .arg("/d")
                    .arg(new_path)
                    .arg("/f")
                    .output()
                    .await
                    .expect("failed to execute process");

                let update_env_result =
                    Self::execute_command(Binary::None, "SETX USERNAME %USERNAME%").await?;

                if !write_reg_result.status.success() || update_env_result.2 != 0 {
                    return Err(anyhow!(
                        "failed to add bin folder to PATH: {}",
                        String::from_utf8(write_reg_result.stderr)?
                    ));
                }

                println!("success");
            }
        }

        Ok(())
    }
}
