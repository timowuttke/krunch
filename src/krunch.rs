use anyhow::{anyhow, Result};
use kube::Client;

pub const MINIKUBE_HOST: &'static str = "k8s.local";
pub const TLS_SECRET: &'static str = "tls";

pub struct Krunch {
    pub client: Client,
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
