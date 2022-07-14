use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use k8s_openapi::api::core::v1::Secret;
use kube::{
    api::{Patch, PatchParams, PostParams},
    Api, Client, ResourceExt,
};
use serde_json::json;

use crate::akapi::{
    token::ViewToken, token_identifier_name, AkApiRoute, AkServer, TEMP_AUTH_TOKEN,
};

use super::crd;

pub async fn reconcile(obj: &crd::Authentik, client: Client) -> Result<()> {
    let instance = obj
        .metadata
        .name
        .clone()
        .ok_or(anyhow!("Missing instance name.".to_string()))?;
    let ns = obj
        .namespace()
        .ok_or(anyhow!("Missing namespace `{}`.", instance.clone()))?;

    // Fetch the token from the Authentik server.
    let mut api = AkServer::connect(&instance, &ns, client.clone()).await?;
    let token = ViewToken::send(
        &mut api,
        TEMP_AUTH_TOKEN,
        token_identifier_name(&instance, "operatortoken"),
    )
    .await?;

    // Create or patch the secret.
    let api: Api<Secret> = Api::namespaced(client, &ns);
    let name = format!("ak-{}-api-operatortoken", instance);
    if let Some(_) = api.get_opt(&name).await? {
        api.patch(
            &name,
            &PatchParams::apply("authentik.ak-operator").force(),
            &Patch::Apply(&build(instance.clone(), obj, token)?),
        )
        .await?;
    } else {
        debug!("Token secret did not exist, creating it now.");
        api.create(
            &PostParams::default(),
            &build(instance.clone(), obj, token)?,
        )
        .await?;
    }

    Ok(())
}

pub async fn cleanup(_obj: &crd::Authentik, _client: Client) -> Result<()> {
    // Note: The secret will automatically be cleaned up by Kubernetes.
    Ok(())
}

fn build(name: String, obj: &crd::Authentik, token: String) -> Result<Secret> {
    let secret: Secret = serde_json::from_value(json!({
        "apiVersion": "v1",
        "kind": "Secret",
        "type": "Opaque",
        "metadata": {
            "name": format!("ak-{}-api-operatortoken", &name),
            "labels": get_labels(name.clone(), obj.spec.image.tag.to_string()),
            "ownerReferences": [{
                "apiVersion": "ak.dany.dev/v1",
                "kind": "Authentik",
                "name": name,
                "uid": obj.uid().expect("Failed to get UID of Authentik."),
                "controller": true
            }]
        },
        "data": {
            "token": base64::encode(token)
        }
    }))?;

    Ok(secret)
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
