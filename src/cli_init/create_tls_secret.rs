use crate::shared::consts::{MINIKUBE_HOST, TLS_SECRET};
use crate::shared::file_folder_paths::{get_binary_path, Binary};
use crate::shared::handle_output;
use anyhow::{anyhow, Result};
use base64::engine::general_purpose;
use base64::Engine;
use k8s_openapi::api::core::v1::Secret;
use kube::api::PostParams;
use kube::{Api, Error};
use std::fs;
use std::fs::File;
use std::io::Read;
use std::process::Command;

pub async fn create_ca_and_install_tls_in_cluster() -> Result<()> {
    install_local_ca()?;
    create_certificate_files()?;
    install_tls_secret().await?;

    Ok(())
}

fn install_local_ca() -> Result<()> {
    let output = Command::new(get_binary_path(Binary::Mkcert)?)
        .arg("--install")
        .output()
        .expect("failed to execute process");

    handle_output(output)?;

    Ok(())
}

async fn install_tls_secret() -> Result<()> {
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

fn create_certificate_files() -> Result<()> {
    let output = Command::new(get_binary_path(Binary::Mkcert)?)
        .arg(MINIKUBE_HOST)
        .output()
        .expect("failed to execute process");

    handle_output(output)?;

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
