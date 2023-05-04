use crate::Krunch;
use anyhow::Result;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::BufReader;

impl Krunch {
    #[cfg(target_family = "unix")]
    pub async fn add_bin_folder_to_path() -> Result<()> {
        let mut profile_path = home::home_dir().unwrap();
        // todo: check for different shell variants, e.g. fn get_unix_file_for_path and make sure file exists
        profile_path.push(".profile");

        let mut profile = OpenOptions::new()
            .read(true)
            .write(true)
            .append(true)
            .open(&profile_path)?;

        let reader = BufReader::new(&profile);
        let mut already_exists = false;
        let bin_folder = Self::get_bin_folder()?;
        for line in reader.lines() {
            let line = line?;
            if line.contains(&bin_folder.display().to_string()) {
                already_exists = true;
                break;
            }
        }

        if already_exists {
            println!("already done");
        } else {
            writeln!(profile, "\n# krunch")?;
            writeln!(profile, "export PATH=\"{}:$PATH\"", bin_folder.display())?;
            println!("success");
        }

        Ok(())
    }

    #[cfg(target_family = "windows")]
    pub async fn add_bin_folder_to_path() -> Result<()> {
        let current_path = Self::get_path_variable()?;

        let bin_folder = Self::get_bin_folder()?.display().to_string();

        if current_path.contains(&bin_folder) {
            println!("already done");
        } else {
            let divider = if current_path.ends_with(";") { "" } else { ";" };
            let new_path = format!("{}{}{};", current_path, divider, bin_folder);
            Self::write_to_environment("Path", new_path)?;

            println!("success");
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

                println!("success");
            }
        };

        Ok(())
    }
}
