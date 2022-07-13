use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use k8s_openapi::api::core::v1::Service;
use kube::{
    api::{Patch, PatchParams, PostParams},
    Api, Client, ResourceExt,
};
use serde_json::json;

use super::{crd, deployment};

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
    if let Some(_) = api.get_opt(&format!("authentik-{}", instance)).await? {
        api.patch(
            &format!("authentik-{}", instance),
            &PatchParams::apply("authentik.ak-operator").force(),
            &Patch::Apply(&build(instance.clone(), obj)?),
        )
        .await?;
    } else {
        api.create(&PostParams::default(), &build(instance.clone(), obj)?)
            .await?;
    }

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
            "labels": get_labels(name.clone(), obj.spec.image.tag.to_string()),
            "ownerReferences": [{
                "apiVersion": "ak.dany.dev/v1",
                "kind": "Authentik",
                "name": name,
                "uid": obj.uid().expect("Failed to get UID of Authentik."),
                "controller": true
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
            "selector": deployment::get_matching_labels(name.clone())
        }
    }))?;

    Ok(service)
}

fn get_labels(instance: String, version: String) -> BTreeMap<String, String> {
    let mut labels = get_matching_labels(instance);
    labels.insert(
        "app.kubernetes.io/created-by".to_string(),
        "authentik-operator".to_string(),
    );
    labels.insert("app.kubernetes.io/version".to_string(), version);

    labels
}

pub fn get_matching_labels(instance: String) -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "app.kubernetes.io/name".to_string(),
            "authentik".to_string(),
        ),
        ("app.kubernetes.io/instance".to_string(), instance),
    ])
}
