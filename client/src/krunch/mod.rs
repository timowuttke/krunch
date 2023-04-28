use anyhow::Result;
use k8s_openapi::api::core::v1::Pod;
use kube::api::{ListParams, ObjectList};
use kube::{Api, Client};
use tokio::process::Child;

mod execute_command;
mod init;
mod mount;

const NAMESPACE: &'static str = "krunch";
const SERVICE_ACCOUNT: &'static str = "krunch";
const CLUSTER_ROLE_BINDING: &'static str = "krunch-gets-cluster-admin";
const DEPLOYMENT: &'static str = "krunch";
const IMAGE: &'static str = "timowuttke/krunch:v1";

pub struct Krunch {
    client: Client,
    mount: Option<Child>,
}

impl Krunch {
    pub async fn new() -> Result<Krunch> {
        let client = Client::try_default().await?;

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
