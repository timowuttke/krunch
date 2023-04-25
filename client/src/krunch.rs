use anyhow::Result;
use futures::{StreamExt, TryStreamExt};
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{Namespace, Pod, ServiceAccount};
use k8s_openapi::api::rbac::v1::ClusterRoleBinding;
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

    pub async fn create_namespace(&self) -> Result<()> {
        let namespace: Namespace = serde_json::from_value(serde_json::json!({
            "apiVersion": "v1",
            "kind": "Namespace",
            "metadata": {
                "labels": {
                    "kubernetes.io/metadata.name": "krunch"
                },
                "name": "krunch",
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

        namespaces
            .create(&PostParams::default(), &namespace)
            .await?;

        Ok(())
    }

    pub async fn create_service_account(&self) -> Result<()> {
        let service_account: ServiceAccount = serde_json::from_value(serde_json::json!({
            "apiVersion": "v1",
            "kind": "ServiceAccount",
            "metadata": {
                "name": "krunch",
                "namespace": "krunch"
            }
        }))?;

        let service_accounts: Api<ServiceAccount> = Api::namespaced(self.client.clone(), "krunch");

        service_accounts
            .create(&PostParams::default(), &service_account)
            .await?;

        Ok(())
    }

    pub async fn create_cluster_role_binding(&self) -> Result<()> {
        let cluster_role_binding: ClusterRoleBinding = serde_json::from_value(serde_json::json!({
            "apiVersion": "rbac.authorization.k8s.io/v1",
            "kind": "ClusterRoleBinding",
            "metadata": {
                "name": "krunch-gets-cluster-admin"
            },
            "subjects": [
                {
                    "kind": "ServiceAccount",
                    "name": "krunch",
                    "namespace": "krunch"
                }
            ],
            "roleRef": {
                "kind": "ClusterRole",
                "name": "cluster-admin",
                "apiGroup": "rbac.authorization.k8s.io"
            }
        }))?;

        let cluster_role_bindings: Api<ClusterRoleBinding> = Api::all(self.client.clone());

        cluster_role_bindings
            .create(&PostParams::default(), &cluster_role_binding)
            .await?;

        Ok(())
    }

    pub async fn create_deployment(&self) -> Result<()> {
        let deployment: Deployment = serde_json::from_value(serde_json::json!({
            "apiVersion": "apps/v1",
            "kind": "Deployment",
            "metadata": {
                "name": "krunch",
                "namespace": "krunch",
                "labels": {
                    "app": "krunch"
                }
            },
            "spec": {
                "replicas": 1,
                "selector": {
                    "matchLabels": {
                        "app": "krunch"
                    }
                },
                "template": {
                    "metadata": {
                        "labels": {
                            "app": "krunch"
                        }
                    },
                    "spec": {
                        "terminationGracePeriodSeconds": 0,
                        "serviceAccountName": "krunch",
                        "containers": [
                            {
                                "name": "krunch",
                                "image": "timowuttke/krunch:latest",
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

        let deployments: Api<Deployment> = Api::namespaced(self.client.clone(), "krunch");

        // Stop on error including a pod already exists or is still being deleted.
        deployments
            .create(&PostParams::default(), &deployment)
            .await?;

        // Wait until the pod is running, otherwise we get 500 error.
        // let wp = WatchParams::default()
        //     .fields("metadata.name=krunch")
        //     .timeout(10);
        // let mut stream = pods.watch(&wp, "0").await?.boxed();
        // while let Some(status) = stream.try_next().await? {
        //     match status {
        //         WatchEvent::Added(o) => {
        //             info!("Added {}", o.name_any());
        //         }
        //         WatchEvent::Modified(o) => {
        //             let s = o.status.as_ref().expect("status exists on pod");
        //             if s.phase.clone().unwrap_or_default() == "Running" {
        //                 info!("Ready to attach to {}", o.name_any());
        //                 break;
        //             }
        //         }
        //         _ => {}
        //     }
        // }

        Ok(())
    }
}

async fn get_pod_name(client: Client) -> String {
    let pods: Api<Pod> = Api::default_namespaced(client);

    let lp = ListParams::default();
    let test: ObjectList<Pod> = pods.list(&lp).await.unwrap();

    test.items
        .iter()
        .find(|e| e.metadata.name.clone().unwrap().starts_with("krunch"))
        .unwrap()
        .metadata
        .name
        .clone()
        .unwrap()
}
