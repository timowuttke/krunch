use crate::shared::file_folder_paths::{get_binary_path, Binary};
use crate::shared::{get_minikube_client, handle_output, power_shell_admin_prompt, TLS_SECRET};
use anyhow::{anyhow, Result};
use k8s_openapi::api::core::v1::Secret;
use kube::api::DeleteParams;
use kube::{Api, Error};
use std::process::Command;

pub async fn remove_ca_and_tls_secret() -> Result<()> {
    let mkcert_path = get_binary_path(Binary::Mkcert)?;

    if mkcert_path.exists() {
        if cfg!(target_family = "unix") {
            remove_local_ca_unix()?;
        } else if cfg!(target_family = "windows") {
            remove_local_ca_windows()?;
        }

        delete_tls_secret().await?;
    } else {
        println!("nothing to do");
    }

    Ok(())
}

fn remove_local_ca_unix() -> Result<()> {
    let output = Command::new("sudo")
        .arg(get_binary_path(Binary::Mkcert)?)
        .arg("--uninstall")
        .output()
        .expect("failed to execute process");

    handle_output(output)?;

    Ok(())
}

fn remove_local_ca_windows() -> Result<()> {
    let command = format!("{} --uninstall", get_binary_path(Binary::Mkcert)?.display());

    let output = power_shell_admin_prompt(command)?;
    handle_output(output)?;

    Ok(())
}

async fn delete_tls_secret() -> Result<()> {
    let client = get_minikube_client().await?;
    let secrets: Api<Secret> = Api::namespaced(client, "default");

    let result = secrets.delete(TLS_SECRET, &DeleteParams::default()).await;

    handle_resource_deletion_result(result)?;

    Ok(())
}

fn handle_resource_deletion_result<T>(result: kube::Result<T, Error>) -> Result<()> {
    match result {
        Ok(_) => println!("success"),
        Err(Error::Api(inner)) => {
            if inner.reason == "NotFound" {
                println!("nothing to do");
            }
        }
        Err(err) => {
            return Err(anyhow!(err));
        }
    }

    Ok(())
}
