use crate::get_minikube_client;
use crate::shared::file_folder_paths::{get_binary_path, Binary};
use crate::shared::handle_output;
use anyhow::Result;
use k8s_openapi::api::core::v1::Node;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::api::{Patch, PatchParams};
use kube::Api;
use serde_json::Value;
use std::collections::BTreeMap;
use std::process::Command;

pub async fn enable_ingress_addon_if_needed() -> Result<()> {
    add_node_primary_label_if_not_exists().await?;
    let status: Value = get_minikbe_addons()?;

    if status["ingress"]["Status"] == "enabled" {
        println!("already done")
    } else {
        enable_minikube_ingress_addon()?;
        println!("success")
    }

    Ok(())
}

async fn add_node_primary_label_if_not_exists() -> Result<()> {
    let client = get_minikube_client().await?;

    let nodes: Api<Node> = Api::all(client);
    let node = nodes.get("minikube").await?;

    match node.metadata.labels {
        Some(labels) if labels.contains_key("minikube.k8s.io/primary") => {}
        _ => {
            let mut new_labels = match node.metadata.labels {
                Some(labels) => labels,
                None => BTreeMap::new(),
            };

            new_labels.insert("minikube.k8s.io/primary".to_string(), "true".to_string());

            let patch = Patch::Apply(Node {
                metadata: ObjectMeta {
                    labels: Some(new_labels),
                    ..Default::default()
                },
                ..Default::default()
            });

            nodes
                .patch("minikube", &PatchParams::apply("krunch"), &patch)
                .await?;
        }
    }

    Ok(())
}

fn enable_minikube_ingress_addon() -> Result<()> {
    let output = Command::new(get_binary_path(Binary::Minikube)?)
        .arg("addons")
        .arg("enable")
        .arg("ingress")
        .output()
        .expect("failed to execute process");

    handle_output(output)?;

    Ok(())
}

fn get_minikbe_addons() -> Result<Value> {
    let output = Command::new(get_binary_path(Binary::Minikube)?)
        .arg("addons")
        .arg("list")
        .arg("--output")
        .arg("json")
        .output()
        .expect("failed to execute process");

    let value: Value = serde_json::from_str(&handle_output(output)?)?;

    Ok(value)
}
