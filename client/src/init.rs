use anyhow::{anyhow, Result};
use futures::{StreamExt, TryStreamExt};
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{ContainerStatus, Namespace, Pod, ServiceAccount};
use k8s_openapi::api::rbac::v1::ClusterRoleBinding;
use std::env;
use std::ops::Add;
use std::{thread, time};

use crate::Krunch;
use kube::api::{ListParams, ObjectList};
use kube::{
    api::{Api, AttachParams, PostParams, ResourceExt, WatchEvent, WatchParams},
    Client, Error,
};
use log::*;

const NAMESPACE: &'static str = "krunch";
const SERVICE_ACCOUNT: &'static str = "krunch";
const CLUSTER_ROLE_BINDING: &'static str = "krunch-gets-cluster-admin";
const DEPLOYMENT: &'static str = "krunch";
const IMAGE: &'static str = "timowuttke/krunch:v1";

impl Krunch {
    pub async fn create_namespace(&self) -> Result<()> {
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

        match namespaces.create(&PostParams::default(), &namespace).await {
            Ok(_) => {}
            Err(Error::Api(inner)) => {
                if inner.reason == "AlreadyExists" {
                    info!("Namespace \"{}\" already exists", NAMESPACE)
                }
            }
            Err(err) => return Err(anyhow!(err)),
        };

        Ok(())
    }

    pub async fn create_service_account(&self) -> Result<()> {
        let service_account: ServiceAccount = serde_json::from_value(serde_json::json!({
            "apiVersion": "v1",
            "kind": "ServiceAccount",
            "metadata": {
                "name": SERVICE_ACCOUNT,
                "namespace": NAMESPACE
            }
        }))?;

        let service_accounts: Api<ServiceAccount> = Api::namespaced(self.client.clone(), NAMESPACE);

        match service_accounts
            .create(&PostParams::default(), &service_account)
            .await
        {
            Ok(_) => {}
            Err(Error::Api(inner)) => {
                if inner.reason == "AlreadyExists" {
                    info!("ServiceAccount \"{}\" already exists", SERVICE_ACCOUNT)
                }
            }
            Err(err) => return Err(anyhow!(err)),
        };

        Ok(())
    }

    pub async fn create_cluster_role_binding(&self) -> Result<()> {
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

        match cluster_role_bindings
            .create(&PostParams::default(), &cluster_role_binding)
            .await
        {
            Ok(_) => {}
            Err(Error::Api(inner)) => {
                if inner.reason == "AlreadyExists" {
                    info!(
                        "ClusterRoleBinding \"{}\" already exists",
                        CLUSTER_ROLE_BINDING
                    )
                }
            }
            Err(err) => return Err(anyhow!(err)),
        }

        Ok(())
    }

    pub async fn create_deployment(&self) -> Result<()> {
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

        match deployments
            .create(&PostParams::default(), &deployment)
            .await
        {
            Ok(_) => {}
            Err(Error::Api(inner)) => {
                if inner.reason == "AlreadyExists" {
                    info!("Deployment \"{}\" already exists", DEPLOYMENT)
                }
            }
            Err(err) => return Err(anyhow!(err)),
        }

        Ok(())
    }

    pub async fn verify_pod_is_healthy(&self) -> Result<()> {
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), NAMESPACE);

        let wp = WatchParams::default()
            .fields("metadata.namespace=krunch")
            .timeout(10);

        let pod: Pod = pods.get("krunch-686fb9db55-mxd8m").await?;

        if Krunch::is_pod_healthy(pod) {
            return Ok(());
        }

        let mut stream = pods.watch(&wp, "0").await?.boxed();
        while let Some(status) = stream.try_next().await? {
            match status {
                WatchEvent::Added(p) => {
                    info!("Added {}", p.name_any());
                }
                WatchEvent::Modified(p) => {
                    if Krunch::is_pod_healthy(p) {
                        info!("Pod is running");
                        break;
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn is_pod_healthy(pod: Pod) -> bool {
        let container_issues: Vec<ContainerStatus> = pod
            .status
            .clone()
            .unwrap()
            .container_statuses
            .unwrap()
            .iter()
            .filter(|s| {
                s.state.clone().unwrap().waiting.is_some()
                    || s.state.clone().unwrap().terminated.is_some()
            })
            .map(|c| c.clone())
            .collect();

        pod.status.clone().unwrap().phase.unwrap() == "Running" && container_issues.is_empty()
    }
}
