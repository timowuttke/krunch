use crate::Krunch;
use anyhow::Result;

impl Krunch {
    pub async fn mkcert(&self) -> Result<()> {
        Self::install_local_ca().await?;
        self.install_certificate_in_cluster().await?;

        Ok(())
    }

    async fn install_certificate_in_cluster(&self) -> Result<()> {
        let (tls_crt, tls_key) = Self::create_certificate().await?;
        self.install_tls_secret(tls_crt, tls_key).await?;

        Ok(())
    }
}
