use crate::Krunch;
use anyhow::Result;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::BufReader;

impl Krunch {
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

    pub async fn point_docker_to_minikube() -> Result<()> {
        if cfg!(target_family = "unix") {
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
}
