use crate::Krunch;
use anyhow::{anyhow, Result};
use std::{fs, str};

// todo: use a tmp path
const MKCERT_FILE_NAME: &'static str = "mkcert_krunch";

impl Krunch {
    pub async fn mkcert(&self) -> Result<()> {
        Krunch::create_os_specific_mkcert_binary()?;

        match Krunch::execute_host_command(format!("./{} --install", MKCERT_FILE_NAME).as_str())
            .await
        {
            Ok((_, stderr, status)) => {
                if status != 0 {
                    return Err(anyhow!("mkcert install failed with: {}", stderr));
                }
                fs::remove_file(MKCERT_FILE_NAME)?;
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
