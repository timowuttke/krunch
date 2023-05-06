use anyhow::{anyhow, Result};
use base64::engine::general_purpose;
use base64::Engine;
use k8s_openapi::api::core::v1::Secret;
use kube::{
    api::{Api, PostParams},
    Error,
};

use crate::cli_init::commands::{
    create_certificate_files, enable_minikube_ingress_addon, get_minikbe_addons,
};

use crate::r#const::{MINIKUBE_HOST, TLS_SECRET};
use serde_json::Value;
use std::fs;
use std::fs::File;
use std::io::Read;

pub fn enabling_ingress_addon() -> Result<()> {
    let status: Value = get_minikbe_addons()?;

    if status["ingress"]["Status"] == "enabled" {
        println!("already done")
    } else {
        enable_minikube_ingress_addon()?;
        println!("success")
    }

    Ok(())
}

pub async fn install_tls_secret() -> Result<()> {
    create_certificate_files()?;

    let mut file = File::open(format!("{}.pem", MINIKUBE_HOST))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let tls_crt = general_purpose::STANDARD.encode(contents);

    let mut file = File::open(format!("{}-key.pem", MINIKUBE_HOST))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let tls_key = general_purpose::STANDARD.encode(contents);

    fs::remove_file(format!("{}-key.pem", MINIKUBE_HOST)).unwrap_or(());
    fs::remove_file(format!("{}.pem", MINIKUBE_HOST)).unwrap_or(());

    let client = get_k8s_client().await?;
    let secrets: Api<Secret> = Api::namespaced(client, "default");

    let secret: Secret = serde_json::from_value(serde_json::json!({
        "apiVersion": "v1",
        "data": {
            "tls.crt": tls_crt,
            "tls.key": tls_key
        },
        "kind": "Secret",
        "metadata": {
            "name": TLS_SECRET,
            "namespace": "default"
        },
        "type": "kubernetes.io/tls"
    }))?;

    //todo: patch
    let result = secrets.create(&PostParams::default(), &secret).await;

    handle_resource_creation_result(result)?;

    Ok(())
}

fn handle_resource_creation_result<T>(result: kube::Result<T, Error>) -> Result<()> {
    match result {
        Ok(_) => println!("success"),
        Err(Error::Api(inner)) => {
            if inner.reason == "AlreadyExists" {
                println!("already done");
            }
        }
        Err(err) => {
            println!("failure");
            return Err(anyhow!(err));
        }
    }

    Ok(())
}

async fn get_k8s_client() -> Result<kube::Client> {
    let client = match kube::Client::try_default().await {
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

    Ok(client)
}
