use std::collections::BTreeMap;

use k8s_openapi::api::networking::v1::Ingress;
use kube::{
    api::{DeleteParams, Patch, PatchParams, PostParams},
    Api, Client, ResourceExt,
};
use serde_json::json;

use crate::ReconcileError;

use super::crd;

pub async fn reconcile(obj: &crd::Authentik, client: Client) -> Result<(), ReconcileError> {
    let instance = obj.metadata.name.clone().ok_or(ReconcileError::InvalidObj(
        "Missing instance name.".to_string(),
    ))?;
    let ns = obj
        .namespace()
        .ok_or(ReconcileError::NoNamespace(instance.clone()))?;
    let name = format!("authentik-{}", instance);
    let api: Api<Ingress> = Api::namespaced(client, &ns);
    let ingress = api.get_opt(&name).await?;

    if let Some(ing) = &obj.spec.ingress {
        // Create or update the ingress.
        if ingress.is_some() {
            api.patch(
                &format!("authentik-{}", instance),
                &PatchParams::apply("authentik.ak-operator").force(),
                &Patch::Apply(&build(instance.clone(), obj, ing)?),
            )
            .await?;
        } else {
            api.create(&PostParams::default(), &build(instance.clone(), obj, ing)?)
                .await?;
        }
    } else {
        if ingress.is_some() {
            // Remove the ingress, as it's no longer in the CRD defined.
            api.delete(&name, &DeleteParams::default()).await?;
        }
    }

    Ok(())
}

pub async fn cleanup(_obj: &crd::Authentik, _client: Client) -> Result<(), ReconcileError> {
    Ok(())
}

fn build(
    name: String,
    obj: &crd::Authentik,
    ing: &crd::AuthentikIngress,
) -> Result<Ingress, ReconcileError> {
    let tls = match &ing.tls {
        Some(tls) => tls
            .iter()
            .map(|t| {
                json!({
                    "hosts": t.hosts,
                    "secretName": t.secret_name,
                })
            })
            .collect(),
        None => Vec::new(),
    };

    let rules: serde_json::Value = ing
        .rules
        .iter()
        .map(|rule| {
            json!({
                "host": rule.host,
                "http": {
                    "paths": rule.paths.iter().map(|path| json!({
                        "pathType": path.path_type,
                        "path": path.path,
                        "backend": {
                            "service": {
                                "name": format!("authentik-{}", name.clone()),
                                "port": {
                                    "name": "http"
                                }
                            }
                        }
                    })).collect::<serde_json::Value>()
                }
            })
        })
        .collect();

    let ingress: Ingress = serde_json::from_value(json!({
        "apiVersion": "networking.k8s.io/v1",
        "kind": "Ingress",
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
            "ingressClassName": ing.class_name,
            "tls": tls,
            "rules": rules,
        }
    }))?;

    Ok(ingress)
}

fn get_labels(instance: String, version: String) -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "app.kubernetes.io/name".to_string(),
            "authentik".to_string(),
        ),
        (
            "app.kubernetes.io/created-by".to_string(),
            "authentik-operator".to_string(),
        ),
        ("app.kubernetes.io/instance".to_string(), instance),
        ("app.kubernetes.io/version".to_string(), version),
    ])
}
