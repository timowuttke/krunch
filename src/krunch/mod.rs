mod commands;
mod download;
mod init;
mod mkcert;
mod urls;

use anyhow::{anyhow, Result};
use kube::Client;

const TLS_SECRET: &'static str = "tls";

pub struct Krunch {
    client: Client,
}

impl Krunch {
    pub async fn new() -> Result<Krunch> {
        let client = match Client::try_default().await {
            Ok(inner) => inner,
            Err(err) => {
                return Err(anyhow!(
                    "failed to load cluster config: {}",
                    err.to_string()
                ));
            }
        };

        match client.apiserver_version().await {
            Ok(inner) => inner,
            Err(_) => {
                return Err(anyhow!(
                    "failed to connect to cluster, is minikube running?"
                ));
            }
        };

        // ToDo: make sure the context is minikube

        Ok(Krunch { client })
    }
}
