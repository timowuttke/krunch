use crate::shared::file_folder_paths::{get_binary_path, Binary};
use crate::shared::{
    get_k8s_client, handle_output, restore_term, save_term, should_continue_as_admin, TLS_SECRET,
};
use anyhow::{anyhow, Result};

use k8s_openapi::api::core::v1::Secret;
use kube::api::DeleteParams;
use kube::{Api, Error};
use std::process::Command;

pub async fn remove_tls_secret() -> Result<()> {
    if !should_continue_as_admin()? {
        return Err(anyhow!("skipped"));
    }
    remove_local_ca()?;
    delete_tls_secret().await?;

    Ok(())
}

fn remove_local_ca() -> Result<()> {
    let mkcert_path = get_binary_path(Binary::Mkcert)?;

    if mkcert_path.exists() {
        save_term()?;

        let output = Command::new("sudo")
            .arg(mkcert_path)
            .arg("-uninstall")
            .output()
            .expect("failed to execute process");

        restore_term(1)?;
        handle_output(output)?;
    }

    Ok(())
}

async fn delete_tls_secret() -> Result<()> {
    let client = get_k8s_client().await?;
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
            println!("failure");
            return Err(anyhow!(err));
        }
    }

    Ok(())
}
