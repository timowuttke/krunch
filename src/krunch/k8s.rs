use crate::krunch::commands::MINIKUBE_HOST;
use crate::krunch::TLS_SECRET;
use crate::Krunch;
use anyhow::{anyhow, Result};
use base64::engine::general_purpose;
use base64::Engine;
use k8s_openapi::api::core::v1::Secret;
use kube::{
    api::{Api, PostParams},
    Error,
};
use serde_json::Value;
use std::fs::File;
use std::io::{Read, Write};
use std::{fs, io};

impl Krunch {
    //todo: move somewhere else
    pub async fn init(&self) -> Result<()> {
        Krunch::download_all().await?;

        print!("{:<30}", "creating TLS secret");
        io::stdout().flush().unwrap();
        Self::install_local_ca()?;
        self.install_tls_secret().await?;

        print!("{:<30}", "enabling ingress addon");
        io::stdout().flush().unwrap();
        self.enabling_ingress_addon().await?;

        Ok(())
    }

    async fn enabling_ingress_addon(&self) -> Result<()> {
        let status: Value = Self::get_minikbe_addons()?;

        if status["ingress"]["Status"] == "enabled" {
            println!("already done")
        } else {
            Self::enable_minikube_ingress_addon()?;
            println!("success")
        }

        Ok(())
    }

    pub async fn install_tls_secret(&self) -> Result<()> {
        Self::create_certificate_files()?;

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
