use crate::Krunch;
use anyhow::{anyhow, Error, Result};
use base64::engine::general_purpose;
use base64::Engine;
use std::fs::File;
use std::io::Read;
use std::{fs, str};

// todo: use a tmp path
const MKCERT_FILE_NAME: &'static str = "mkcert_krunch";
const MKCERT_HOST: &'static str = "k8s.local";

impl Krunch {
    pub async fn mkcert(&self) -> Result<()> {
        let result = async {
            Self::create_os_specific_mkcert_binary()?;
            Self::install_local_ca().await?;
            self.install_certificate_in_cluster().await?;
            Ok::<(), Error>(())
        }
        .await;

        Self::clean_up().await?;
        result?;

        Ok(())
    }

    async fn install_certificate_in_cluster(&self) -> Result<()> {
        match Krunch::execute_host_command(
            format!("./{} {}", MKCERT_FILE_NAME, MKCERT_HOST).as_str(),
        )
        .await
        {
            Ok((_, stderr, status)) => {
                if status != 0 {
                    return Err(anyhow!("mkcert cert creation failed with: {}", stderr));
                }
            }
            Err(err) => {
                return Err(anyhow!("mkcert cert creation failed with: {}", err));
            }
        };

        let mut file = File::open(format!("{}.pem", MKCERT_HOST))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let tls_crt = general_purpose::STANDARD.encode(contents);

        let mut file = File::open(format!("{}-key.pem", MKCERT_HOST))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let tls_key = general_purpose::STANDARD.encode(contents);

        self.install_tls_secret(tls_crt, tls_key).await?;

        Ok(())
    }

    async fn install_local_ca() -> Result<()> {
        match Krunch::execute_host_command(format!("./{} --install", MKCERT_FILE_NAME).as_str())
            .await
        {
            Ok((_, stderr, status)) => {
                if status != 0 {
                    return Err(anyhow!("mkcert install failed with: {}", stderr));
                }
            }
            Err(err) => {
                return Err(anyhow!("mkcert install failed with: {}", err));
            }
        };

        Ok(())
    }

    fn create_os_specific_mkcert_binary() -> Result<()> {
        let binary;

        #[cfg(target_arch = "x86_64")]
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::fs::PermissionsExt;

            binary = include_bytes!("../mkcert/mkcert-v1.4.4-linux-amd64");
            std::fs::write(MKCERT_FILE_NAME, binary)?;
            fs::set_permissions(MKCERT_FILE_NAME, fs::Permissions::from_mode(0o755))?;
        }

        #[cfg(target_arch = "x86_64")]
        #[cfg(target_os = "macos")]
        {
            use std::os::unix::fs::PermissionsExt;

            binary = include_bytes!("../mkcert/mkcert-v1.4.4-darwin-amd64");
            std::fs::write(MKCERT_FILE_NAME, binary)?;
            fs::set_permissions(MKCERT_FILE_NAME, fs::Permissions::from_mode(0o755))?;
        }

        #[cfg(target_arch = "arm")]
        #[cfg(target_os = "macos")]
        {
            use std::os::unix::fs::PermissionsExt;

            binary = include_bytes!("../mkcert/mkcert-v1.4.4-darwin-arm64");
            std::fs::write(MKCERT_FILE_NAME, binary)?;
            fs::set_permissions(MKCERT_FILE_NAME, fs::Permissions::from_mode(0o755))?;
        }

        #[cfg(target_arch = "x86_64")]
        #[cfg(target_os = "windows")]
        {
            binary = include_bytes!("../mkcert/mkcert-v1.4.4-windows-amd64.exe");
            std::fs::write(MKCERT_FILE_NAME, binary)?;
        }

        Ok(())
    }

    async fn clean_up() -> Result<()> {
        fs::remove_file(MKCERT_FILE_NAME)?;
        fs::remove_file(format!("{}-key.pem", MKCERT_HOST))?;
        fs::remove_file(format!("{}.pem", MKCERT_HOST))?;

        Ok(())
    }
}
