use crate::krunch::commands::Binary;
use crate::krunch::TLS_SECRET;
use crate::Krunch;
use anyhow::{anyhow, Result};
use k8s_openapi::api::core::v1::Secret;
use kube::{
    api::{Api, PostParams},
    Error,
};
use serde_json::Value;
use std::io;
use std::io::Write;

impl Krunch {
    pub async fn init(&self) -> Result<()> {
        Krunch::download_all().await?;

        print!("{:<30}", "creating TLS secret");
        io::stdout().flush().unwrap();
        self.mkcert().await?;

        print!("{:<30}", "enabling ingress addon");
        io::stdout().flush().unwrap();
        self.enabling_ingress_addon().await?;

        Ok(())
    }

    async fn enabling_ingress_addon(&self) -> Result<()> {
        let status: Value = serde_json::from_str(
            &*Krunch::execute_command(Binary::Minikube, "addons list --output json")
                .await?
                .0,
        )?;

        if status["ingress"]["Status"] == "enabled" {
            println!("already done")
        } else {
            Krunch::execute_command(Binary::Minikube, "addons enable ingress").await?;
            println!("success")
        }

        Ok(())
    }

    pub async fn install_tls_secret(&self, tls_crt: String, tls_key: String) -> Result<()> {
        let secrets: Api<Secret> = Api::namespaced(self.client.clone(), "default");

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

        Krunch::handle_resource_creation_result(result)?;

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
}
