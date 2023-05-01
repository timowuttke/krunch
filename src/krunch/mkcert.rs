use crate::krunch::command::Binary;
use crate::Krunch;
use anyhow::{anyhow, Error, Result};
use base64::engine::general_purpose;
use base64::Engine;
use std::fs::File;
use std::io::Read;
use std::{fs, str};

const MKCERT_HOST: &'static str = "k8s.local";

impl Krunch {
    pub async fn mkcert(&self) -> Result<()> {
        let result = async {
            Self::install_local_ca().await?;
            self.install_certificate_in_cluster().await?;
            Ok::<(), Error>(())
        }
        .await;

        Self::clean_up();
        result?;

        Ok(())
    }

    async fn install_certificate_in_cluster(&self) -> Result<()> {
        match Krunch::execute_command(Binary::Mkcert, MKCERT_HOST).await {
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
        match Krunch::execute_command(Binary::Mkcert, "--install").await {
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

    fn clean_up() {
        fs::remove_file(format!("{}-key.pem", MKCERT_HOST));
        fs::remove_file(format!("{}.pem", MKCERT_HOST));
    }
}
