use anyhow::{anyhow, Result};
use k8s_openapi::api::core::v1::Pod;
use kube::api::{ListParams, ObjectList};
use kube::{Api, Client};
use tokio::process::Child;

mod bomb;
mod execute_command;
mod init;
mod mkcert;
mod mount;

const NAMESPACE: &'static str = "krunch";
const SERVICE_ACCOUNT: &'static str = "krunch";
const CLUSTER_ROLE_BINDING: &'static str = "krunch-gets-cluster-admin";
const DEPLOYMENT: &'static str = "krunch";
const IMAGE: &'static str = "timowuttke/krunch:0.1.0";
const TLS_SECRET: &'static str = "tls";

pub struct Krunch {
    client: Client,
    mount: Option<Child>,
}

impl Krunch {
    pub async fn new() -> Result<Krunch> {
        let client = match Client::try_default().await {
            Ok(inner) => inner,
            Err(_) => {
                return Err(anyhow!(
                    "minikube is not running, consider starting it with \"minikube start\""
                ));
            }
        };
        // ToDo: make sure the context is minikube

        Ok(Krunch {
            client,
            mount: None,
        })
    }

    pub async fn get_krunch_pod_name(&self) -> Option<String> {
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), NAMESPACE);

        let lp = ListParams::default().labels(&format!("app={}", DEPLOYMENT));
        let test: ObjectList<Pod> = pods.list(&lp).await.unwrap();

        return if test.items.is_empty() {
            None
        } else {
            Some(
                test.items
                    .iter()
                    .next()
                    .unwrap()
                    .metadata
                    .name
                    .clone()
                    .unwrap(),
            )
        };
    }
}
