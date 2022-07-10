use std::sync::Arc;

use k8s_openapi::api::apps::v1::Deployment;
use kube::{api::PostParams, Api, Client, ResourceExt};
use serde_json::json;

use crate::ReconcileError;

use super::crd;

pub async fn reconcile(obj: Arc<crd::Authentik>, client: Client) -> Result<(), ReconcileError> {
    // Todo, use a more type-safe method of creating the deployment.
    let deploy: Deployment = serde_json::from_value(json!({
        "apiVersion": "apps/v1",
        "kind": "Deployment",
        "metadata": {
            "name": "testing",
            "labels": {
                "app": "authentik"
            }
        },
        "spec": {
            "replicas": 1,
            "selector": {
                "matchLabels": {
                    "app": "authentik"
                }
            },
            "template": {
                "metadata": {
                    "labels": {
                        "app": "authentik"
                    }
                },
                "spec": {
                    "containers": [{
                        "name": "yeet",
                        "image": "ghcr.io/goauthentik/server:2022.7.2",
                        "ports": [
                            { "name": "http", "containerPort": 9000, "protocol": "TCP" }
                        ]
                    }]
                }
            }
        }
    }))?;

    let ns = obj
        .namespace()
        .ok_or(ReconcileError::NoNamespace(obj.name_any()))?;
    let deployments: Api<Deployment> = Api::namespaced(client, &ns);

    // Todo: Update exsisting deployment when the deployment already exists.
    let pp = PostParams::default();
    deployments.create(&pp, &deploy).await?;
    // let pp = PatchParams::default();
    // let _o = deployments
    //     .patch("testing", &pp, &Patch::Merge(&deploy))
    //     .await?;

    Ok(())
}

pub async fn cleanup(_obj: Arc<crd::Authentik>, _client: Client) -> Result<(), ReconcileError> {
    // Todo: the actual cleanup.
    todo!();
}
