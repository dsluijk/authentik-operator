use anyhow::{anyhow, Result};
use k8s_openapi::api::core::v1::Secret;
use kube::{
    api::{Patch, PatchParams},
    Api, Client, ResourceExt,
};
use serde_json::json;

use crate::akapi::{
    auth::{get_valid_secret_token, get_valid_token},
    token::ViewToken,
    token_identifier_name, AkApiRoute, AkClient,
};

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

    // Check if the current secret stored is valid.
    // This is to surpress the logs in Authentik.
    if get_valid_secret_token(client.clone(), &ns, &instance)
        .await?
        .is_some()
    {
        return Ok(());
    }

    // Get the token.
    let api_key = get_valid_token(client.clone(), &ns, &instance).await?;
    let ak = AkClient::new(&api_key, &instance, &ns)?;

    // Fetch the token from the Authentik server.
    let token = ViewToken::send(&ak, token_identifier_name(&instance, "operatortoken")).await?;

    // Create or patch the secret.
    let api: Api<Secret> = Api::namespaced(client, &ns);
    let name = format!("ak-{}-api-operatortoken", instance);
    api.patch(
        &name,
        &PatchParams::apply("authentik.ak-operator"),
        &Patch::Apply(&build(instance.clone(), obj, token)?),
    )
    .await?;

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
            "labels": labels::get_labels(name.clone(), obj.spec.image.tag.to_string(), "secret".to_string()),
            "ownerReferences": [{
                "apiVersion": "ak.dany.dev/v1",
                "kind": "Authentik",
                "name": name,
                "uid": obj.uid().expect("Failed to get UID of Authentik."),
                "controller": true,
            }]
        },
        "stringData": {
            "token": token
        }
    }))?;

    Ok(secret)
}
