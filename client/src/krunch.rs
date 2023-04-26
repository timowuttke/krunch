use anyhow::{anyhow, Result};
use k8s_openapi::api::core::v1::Pod;
use kube::api::{ListParams, ObjectList};
use kube::{
    api::{Api, AttachParams},
    Client,
};
use std::env;
use std::ops::Add;

use crate::{Krunch, DEPLOYMENT, NAMESPACE};

impl Krunch {
    pub async fn new() -> Result<Krunch> {
        let client = Client::try_default().await?;

        Ok(Krunch { client })
    }

    pub fn create_command(&self) -> Result<Vec<String>> {
        let mut args: Vec<String> = env::args().collect();
        args.remove(0);

        let params = args
            .iter()
            .fold(String::new(), |acc, x| acc.add(x).add(" "));

        let command = vec!["sh".to_string(), "-c".to_string(), params];

        Ok(command)
    }

    pub async fn execute_generic_command(&self, command: Vec<String>) -> Result<()> {
        let pod_name: String = match self.get_pod_name().await {
            None => return Err(anyhow!("Pod not found")),
            Some(inner) => inner,
        };

        let ap = AttachParams::default();
        let pods: Api<Pod> = Api::default_namespaced(self.client.clone());
        let mut attached = pods.exec(&pod_name, command, &ap).await?;

        let mut stdout_reader = attached.stdout().unwrap();
        let mut stdout = tokio::io::stdout();

        tokio::spawn(async move {
            tokio::io::copy(&mut stdout_reader, &mut stdout)
                .await
                .unwrap();
        });
        let status = attached.take_status().unwrap().await.unwrap();

        println!("{:?}", status);

        Ok(())
    }

    pub async fn get_pod_name(&self) -> Option<String> {
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
