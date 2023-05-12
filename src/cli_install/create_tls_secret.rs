use crate::shared::file_folder_paths::{get_binary_path, Binary};
use crate::shared::{
    get_minikube_client, handle_output, restore_term, save_term, should_continue_as_admin,
    MINIKUBE_HOST, TLS_SECRET,
};
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
    if !should_continue_as_admin()? {
        return Err(anyhow!("skipped"));
    }
    install_local_ca()?;
    create_certificate_files()?;
    install_tls_secret().await?;

    Ok(())
}

fn install_local_ca() -> Result<()> {
    save_term()?;

    let output = Command::new(get_binary_path(Binary::Mkcert)?)
        .arg("--install")
        .output()
        .expect("failed to execute process");

    restore_term(1)?;
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

    let client = get_minikube_client().await?;
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
