use crate::krunch::{CLUSTER_ROLE_BINDING, DEPLOYMENT, IMAGE, NAMESPACE, SERVICE_ACCOUNT};
use crate::Krunch;
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine as _};
use futures::{StreamExt, TryStreamExt};
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{ContainerStatus, Namespace, Pod, Secret, ServiceAccount};
use k8s_openapi::api::rbac::v1::ClusterRoleBinding;
use kube::{
    api::{Api, PostParams, WatchEvent, WatchParams},
    Error,
};
use rcgen::generate_simple_self_signed;
use serde_json::Value;
use std::io;
use std::io::Write;

impl Krunch {
    pub async fn init(&self) -> Result<()> {
        print!("{:<30}", "enabling ingress addon");
        io::stdout().flush().unwrap();
        self.enabling_ingress_addon().await?;

        print!("{:<30}", "creating TLS secret");
        io::stdout().flush().unwrap();
        self.install_tls_secret().await?;

        print!("{:<30}", "creating namespace");
        io::stdout().flush().unwrap();
        self.create_namespace().await?;

        print!("{:<30}", "creating service account");
        io::stdout().flush().unwrap();
        self.create_service_account().await?;

        print!("{:<30}", "creating role binding");
        io::stdout().flush().unwrap();
        self.create_cluster_role_binding().await?;

        print!("{:<30}", "creating deployment");
        io::stdout().flush().unwrap();
        self.create_deployment().await?;

        print!("{:<30}", "verifying pod is healthy");
        io::stdout().flush().unwrap();
        self.wait_for_pod_to_be_healthy().await?;
        println!("done");

        Ok(())
    }

    async fn enabling_ingress_addon(&self) -> Result<()> {
        let status: Value = serde_json::from_str(
            &*Krunch::execute_host_command("minikube addons list --output json")
                .await?
                .0,
        )?;

        if status["ingress"]["Status"] == "enabled" {
            println!("already enabled")
        } else {
            Krunch::execute_host_command("minikube addons enable ingress").await?;
            println!("done")
        }

        Ok(())
    }

    async fn create_namespace(&self) -> Result<()> {
        let namespace: Namespace = serde_json::from_value(serde_json::json!({
            "apiVersion": "v1",
            "kind": "Namespace",
            "metadata": {
                "labels": {
                    "kubernetes.io/metadata.name": NAMESPACE
                },
                "name": NAMESPACE,
            },
            "spec": {
                "finalizers": [
                    "kubernetes"
                ]
            },
            "status": {
                "phase": "Active"
            }
        }))?;

        let namespaces: Api<Namespace> = Api::all(self.client.clone());

        let result = namespaces.create(&PostParams::default(), &namespace).await;
        Krunch::handle_resource_creation_result(result)?;

        Ok(())
    }

    async fn create_service_account(&self) -> Result<()> {
        let service_account: ServiceAccount = serde_json::from_value(serde_json::json!({
            "apiVersion": "v1",
            "kind": "ServiceAccount",
            "metadata": {
                "name": SERVICE_ACCOUNT,
                "namespace": NAMESPACE
            }
        }))?;

        let service_accounts: Api<ServiceAccount> = Api::namespaced(self.client.clone(), NAMESPACE);

        let result = service_accounts
            .create(&PostParams::default(), &service_account)
            .await;

        Krunch::handle_resource_creation_result(result)?;

        Ok(())
    }

    async fn create_cluster_role_binding(&self) -> Result<()> {
        let cluster_role_binding: ClusterRoleBinding = serde_json::from_value(serde_json::json!({
            "apiVersion": "rbac.authorization.k8s.io/v1",
            "kind": "ClusterRoleBinding",
            "metadata": {
                "name": CLUSTER_ROLE_BINDING
            },
            "subjects": [
                {
                    "kind": "ServiceAccount",
                    "name": SERVICE_ACCOUNT,
                    "namespace": NAMESPACE
                }
            ],
            "roleRef": {
                "kind": "ClusterRole",
                "name": "cluster-admin",
                "apiGroup": "rbac.authorization.k8s.io"
            }
        }))?;

        let cluster_role_bindings: Api<ClusterRoleBinding> = Api::all(self.client.clone());

        let result = cluster_role_bindings
            .create(&PostParams::default(), &cluster_role_binding)
            .await;

        Krunch::handle_resource_creation_result(result)?;

        Ok(())
    }

    async fn create_deployment(&self) -> Result<()> {
        let deployment: Deployment = serde_json::from_value(serde_json::json!({
            "apiVersion": "apps/v1",
            "kind": "Deployment",
            "metadata": {
                "name": DEPLOYMENT,
                "namespace": NAMESPACE,
                "labels": {
                    "app": DEPLOYMENT
                }
            },
            "spec": {
                "replicas": 1,
                "selector": {
                    "matchLabels": {
                        "app": DEPLOYMENT
                    }
                },
                "template": {
                    "metadata": {
                        "labels": {
                            "app": DEPLOYMENT
                        }
                    },
                    "spec": {
                        "terminationGracePeriodSeconds": 0,
                        "serviceAccountName": SERVICE_ACCOUNT,
                        "containers": [
                            {
                                "name": DEPLOYMENT,
                                "image": IMAGE,
                                "volumeMounts": [
                                    {
                                        "mountPath": "/var/run",
                                        "name": "docker-sock"
                                    },
                                    {
                                        "mountPath": "/krunch",
                                        "name": "krunch"
                                    }
                                ]
                            }
                        ],
                        "volumes": [
                            {
                                "name": "docker-sock",
                                "hostPath": {
                                    "path": "/var/run"
                                }
                            },
                            {
                                "name": "krunch",
                                "hostPath": {
                                    "path": "/krunch"
                                }
                            }
                        ]
                    }
                }
            }
        }))?;

        let deployments: Api<Deployment> = Api::namespaced(self.client.clone(), NAMESPACE);

        let result = deployments
            .create(&PostParams::default(), &deployment)
            .await;

        Krunch::handle_resource_creation_result(result)?;

        Ok(())
    }

    async fn install_tls_secret(&self) -> Result<()> {
        let subject_alt_names = vec!["k8s.local".to_string()];

        let cert = generate_simple_self_signed(subject_alt_names).unwrap();
        let tls_crt = general_purpose::STANDARD.encode(cert.serialize_pem().unwrap());
        let tls_key = general_purpose::STANDARD.encode(cert.serialize_private_key_pem());

        let secrets: Api<Secret> = Api::namespaced(self.client.clone(), "default");

        let secret: Secret = serde_json::from_value(serde_json::json!({
            "apiVersion": "v1",
            "data": {
                "tls.crt": tls_crt,
                "tls.key": tls_key
            },
            "kind": "Secret",
            "metadata": {
                "name": "tls-by-krunch",
                "namespace": "default"
            },
            "type": "kubernetes.io/tls"
        }))?;

        let result = secrets.create(&PostParams::default(), &secret).await;

        Krunch::handle_resource_creation_result(result)?;

        Ok(())
    }

    pub async fn wait_for_pod_to_be_healthy(&self) -> Result<()> {
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), NAMESPACE);

        let wp = WatchParams::default()
            .fields("metadata.namespace=krunch")
            .timeout(20);

        if let Some(pod_name) = self.get_krunch_pod_name().await {
            let pod: Pod = pods.get(pod_name.as_str()).await?;

            if Krunch::is_pod_healthy(pod) {
                return Ok(());
            }
        }

        let mut stream = pods.watch(&wp, "0").await?.boxed();
        while let Some(status) = stream.try_next().await? {
            match status {
                WatchEvent::Added(p) | WatchEvent::Modified(p) => {
                    if Krunch::is_pod_healthy(p) {
                        return Ok(());
                    }
                }
                _ => {}
            }
        }

        Err(anyhow!("timeout waiting for pod to be ready"))
    }

    fn is_pod_healthy(pod: Pod) -> bool {
        let pod_status = match pod.status {
            None => return false,
            Some(inner) => inner.clone(),
        };

        let pod_phase = match pod_status.phase {
            None => return false,
            Some(inner) => inner.clone(),
        };

        let container_statuses = match pod_status.container_statuses {
            None => return false,
            Some(inner) => inner.clone(),
        };

        let container_issues: Vec<ContainerStatus> = container_statuses
            .iter()
            .filter(|s| {
                s.state.is_none()
                    || s.state.clone().unwrap().waiting.is_some()
                    || s.state.clone().unwrap().terminated.is_some()
            })
            .map(|c| c.clone())
            .collect();

        pod_phase == "Running" && container_issues.is_empty()
    }

    fn handle_resource_creation_result<T>(result: kube::Result<T, Error>) -> Result<()> {
        match result {
            Ok(_) => println!("done"),
            Err(Error::Api(inner)) => {
                if inner.reason == "AlreadyExists" {
                    println!("already exists");
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
