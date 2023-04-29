use crate::Krunch;
use anyhow::{anyhow, Result};
use base64::engine::general_purpose;
use base64::Engine;
use std::fs::File;
use std::io::Read;
use std::{fs, str};

// todo: use a tmp path
const MKCERT_FILE_NAME: &'static str = "mkcert_krunch";

impl Krunch {
    pub async fn mkcert(&self) -> Result<()> {
        Self::create_os_specific_mkcert_binary()?;
        Self::install_local_ca().await?;
        fs::remove_file(MKCERT_FILE_NAME)?;
        self.install_certificate_in_cluster().await?;

        Ok(())
    }

    async fn install_certificate_in_cluster(&self) -> Result<()> {
        match Krunch::execute_host_command(format!("./{} k8s.local", MKCERT_FILE_NAME).as_str())
            .await
        {
            Ok((_, stderr, status)) => {
                if status != 0 {
                    fs::remove_file(MKCERT_FILE_NAME)?;
                    return Err(anyhow!("mkcert cert creation failed with: {}", stderr));
                }
            }
            Err(err) => {
                fs::remove_file(MKCERT_FILE_NAME)?;
                return Err(anyhow!("mkcert cert creation failed with: {}", err));
            }
        };

        let mut file = File::open("k8s.local.pem")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let tls_crt = general_purpose::STANDARD.encode(contents);

        let mut file = File::open("k8s.local-key.pem")?;
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
                    fs::remove_file(MKCERT_FILE_NAME)?;
                    return Err(anyhow!("mkcert install failed with: {}", stderr));
                }
            }
            Err(err) => {
                fs::remove_file(MKCERT_FILE_NAME)?;
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
            use std::os::windows::fs::PermissionsExt;

            binary = include_bytes!("../mkcert/mkcert-v1.4.4-windows-amd64.exe");
            std::fs::write(MKCERT_FILE_NAME, binary)?;
            fs::set_permissions(MKCERT_FILE_NAME, fs::Permissions::from_mode(0o755))?;
        }

        Ok(())
    }
}
