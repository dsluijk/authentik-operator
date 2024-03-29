use anyhow::{anyhow, Result};
use k8s_openapi::api::networking::v1::Ingress;
use kube::{
    api::{DeleteParams, Patch, PatchParams},
    Api, Client, ResourceExt,
};
use serde_json::{json, Value};

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
    let name = format!("authentik-{}", instance);
    let api: Api<Ingress> = Api::namespaced(client, &ns);
    let ingress = api.get_opt(&name).await?;

    if let Some(ing) = &obj.spec.ingress {
        // Create or update the ingress.
        api.patch(
            &format!("authentik-{}", instance),
            &PatchParams::apply("authentik.ak-operator").force(),
            &Patch::Apply(build(instance.clone(), obj, ing)),
        )
        .await?;
    } else {
        if ingress.is_some() {
            // Remove the ingress, as it's no longer in the CRD defined.
            api.delete(&name, &DeleteParams::default()).await?;
        }
    }

    Ok(())
}

pub async fn cleanup(_obj: &crd::Authentik, _client: Client) -> Result<()> {
    Ok(())
}

fn build(name: String, obj: &crd::Authentik, ing: &crd::AuthentikIngress) -> Value {
    let tls = ing
        .tls
        .iter()
        .map(|tls| {
            json!({
                "hosts": tls.hosts,
                "secretName": tls.secret_name,
            })
        })
        .collect::<Vec<serde_json::Value>>();

    let rules = ing
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
        .collect::<Vec<serde_json::Value>>();

    // Convert empty rule lists to a None value.
    // This is to prevent a weird issue where the ingress keep triggering new reconcilidations.
    let tls = if !tls.is_empty() { Some(tls) } else { None };
    let rules = if !rules.is_empty() { Some(rules) } else { None };

    json!({
        "apiVersion": "networking.k8s.io/v1",
        "kind": "Ingress",
        "metadata": {
            "name": format!("authentik-{}", name.clone()),
            "labels": labels::get_labels(name.clone(), obj.spec.image.tag.to_string(), "ingress".to_string()),
            "ownerReferences": [{
                "apiVersion": "ak.dany.dev/v1",
                "kind": "Authentik",
                "name": name,
                "uid": obj.uid().expect("Failed to get UID of Authentik."),
                "controller": true,
            }]
        },
        "spec": {
            "ingressClassName": ing.class_name,
            "rules": rules,
            "tls": tls,
        }
    })
}
