use anyhow::Result;
use futures::{StreamExt, TryStreamExt};
use k8s_openapi::api::core::v1::Pod;
use std::env;
use std::ops::Add;

use kube::api::{ListParams, ObjectList};
use kube::{
    api::{Api, AttachParams, PostParams, ResourceExt, WatchEvent, WatchParams},
    Client,
};
use log::*;

pub struct Krunch {
    client: Client,
}

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
        let pod_name: String = get_pod_name(self.client.clone()).await;

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

        info!("{:?}", status);

        Ok(())
    }
}

async fn get_pod_name(client: Client) -> String {
    let pods: Api<Pod> = Api::default_namespaced(client);

    let lp = ListParams::default();
    let test: ObjectList<Pod> = pods.list(&lp).await.unwrap();

    test.items
        .iter()
        .find(|e| e.metadata.name.clone().unwrap().starts_with("playground"))
        .unwrap()
        .metadata
        .name
        .clone()
        .unwrap()
}

async fn create_pod(client: Client) -> Result<()> {
    let p: Pod = serde_json::from_value(serde_json::json!({
        "apiVersion": "v1",
        "kind": "Pod",
        "metadata": { "name": "example" },
        "spec": {
            "containers": [{
                "name": "example",
                "image": "alpine",
                // Do nothing
                "command": ["tail", "-f", "/dev/null"],
            }],
        }
    }))?;

    let pods: Api<Pod> = Api::default_namespaced(client);
    // Stop on error including a pod already exists or is still being deleted.
    pods.create(&PostParams::default(), &p).await?;

    // Wait until the pod is running, otherwise we get 500 error.
    let wp = WatchParams::default()
        .fields("metadata.name=example")
        .timeout(10);
    let mut stream = pods.watch(&wp, "0").await?.boxed();
    while let Some(status) = stream.try_next().await? {
        match status {
            WatchEvent::Added(o) => {
                info!("Added {}", o.name_any());
            }
            WatchEvent::Modified(o) => {
                let s = o.status.as_ref().expect("status exists on pod");
                if s.phase.clone().unwrap_or_default() == "Running" {
                    info!("Ready to attach to {}", o.name_any());
                    break;
                }
            }
            _ => {}
        }
    }

    Ok(())
}
