use anyhow::{anyhow, Result};
use k8s_openapi::api::core::v1::Service;
use kube::{
    api::{Patch, PatchParams},
    Api, Client, ResourceExt,
};
use serde_json::json;

use super::{crd, labels};

pub async fn reconcile(obj: &crd::Authentik, client: Client) -> Result<()> {
    let instance = obj
        .metadata
        .name
        .clone()
        .ok_or(anyhow!("Missing instance name.".to_string()))?;
    let ns = obj
        .namespace()
        .ok_or(anyhow!("Missing namespace `{}`.", instance.clone()))?;

    let api: Api<Service> = Api::namespaced(client, &ns);
    api.patch(
        &format!("authentik-{}", instance),
        &PatchParams::apply("authentik.ak-operator").force(),
        &Patch::Apply(&build(instance.clone(), obj)?),
    )
    .await?;

    Ok(())
}

pub async fn cleanup(_obj: &crd::Authentik, _client: Client) -> Result<()> {
    Ok(())
}

fn build(name: String, obj: &crd::Authentik) -> Result<Service> {
    let service: Service = serde_json::from_value(json!({
        "apiVersion": "v1",
        "kind": "Service",
        "metadata": {
            "name": format!("authentik-{}", name.clone()),
            "labels": labels::get_labels(name.clone(), obj.spec.image.tag.to_string(), "service".to_string()),
            "ownerReferences": [{
                "apiVersion": "ak.dany.dev/v1",
                "kind": "Authentik",
                "name": name,
                "uid": obj.uid().expect("Failed to get UID of Authentik.")
            }]
        },
        "spec": {
            "type": "ClusterIP",
            "ports": [{
                "name": "http",
                "port": 80,
                "targetPort": "http",
                "protocol": "TCP"
            }],
            "selector": labels::get_matching_labels(name.clone(), "server".to_string())
        }
    }))?;

    Ok(service)
}
