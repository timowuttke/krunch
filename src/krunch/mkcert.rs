use crate::Krunch;
use anyhow::Result;

impl Krunch {
    pub async fn mkcert(&self) -> Result<()> {
        Self::install_local_ca()?;
        self.install_tls_secret().await?;

        Ok(())
    }
}
