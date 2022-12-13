use anyhow::{anyhow, Result};
use base64::encode;
use k8s_openapi::api::core::v1::Secret;
use kube::{
    api::{Patch, PatchParams},
    Api, Client, ResourceExt,
};
use serde_json::{json, Value};

use crate::akapi::{
    auth::get_valid_token,
    provider::{FindOAuthProvider, FindOAuthProviderBody},
    types::OAuthProvider,
    AkApiRoute, AkClient,
};

use super::{crd, labels};

pub async fn reconcile(obj: &crd::AuthentikOAuthProvider, client: Client) -> Result<()> {
    let instance = obj.spec.authentik_instance.to_string();
    let ns = obj
        .namespace()
        .ok_or(anyhow!("Missing namespace `{}`.", instance.clone()))?;

    // Get the API key.
    let api_key = get_valid_token(client.clone(), &ns, &instance).await?;
    let ak = AkClient::new(&api_key, &instance, &ns)?;

    // Get the provider from the API.
    let providers = FindOAuthProvider::send(
        &ak,
        FindOAuthProviderBody {
            name: Some(obj.spec.name.clone()),
        },
    )
    .await?;

    let provider = providers
        .iter()
        .find(|&provider| provider.name == obj.spec.name)
        .ok_or(anyhow!("Unable to find the provider `{}`.", obj.spec.name))?;

    // Patch the secret.
    let secret_api: Api<Secret> = Api::namespaced(client.clone(), &ns);
    let secret_name = format!("ak-{}-oauth-{}", instance, obj.name_any());
    secret_api
        .patch(
            &secret_name,
            &PatchParams::apply("authentik.ak-operator").force(),
            &Patch::Apply(build(obj, &secret_name, provider)),
        )
        .await?;

    info!("Updated the OAuth provider `{}`.", obj.name_any());

    Ok(())
}

pub async fn cleanup(_obj: &crd::AuthentikOAuthProvider, _client: Client) -> Result<()> {
    // Note: The secret will automatically be cleaned up by Kubernetes.
    Ok(())
}

fn build(obj: &crd::AuthentikOAuthProvider, secret_name: &str, provider: &OAuthProvider) -> Value {
    let labels = labels::get_labels(
        obj.spec.authentik_instance.to_string(),
        "secret".to_string(),
    );

    let client_type = provider
        .client_type
        .as_ref()
        .map(|c| encode(c.to_string()))
        .unwrap_or("".to_string());
    let client_id = provider
        .client_id
        .as_ref()
        .map(|c| encode(c))
        .unwrap_or("".to_string());
    let client_secret = provider
        .client_secret
        .as_ref()
        .map(|c| encode(c))
        .unwrap_or("".to_string());
    let redirect_uris = provider
        .redirect_uris
        .as_ref()
        .map(|c| encode(c))
        .unwrap_or("".to_string());

    json!({
        "apiVersion": "v1",
        "kind": "Secret",
        "type": "Opaque",
        "metadata": {
            "name": secret_name,
            "labels": labels,
            "ownerReferences": [{
                "apiVersion": "ak.dany.dev/v1",
                "kind": "AuthentikOAuthProvider",
                "name": obj.name_any(),
                "uid": obj.uid().expect("Failed to get UID of Authentik."),
                "controller": true,
            }]
        },
        "data": {
            "clientType": client_type,
            "clientId": client_id,
            "clientSecret": client_secret,
            "redirectUris": redirect_uris
        }
    })
}
