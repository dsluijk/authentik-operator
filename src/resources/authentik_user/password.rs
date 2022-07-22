use anyhow::{anyhow, Result};
use k8s_openapi::api::core::v1::Secret;
use kube::{api::PostParams, Api, Client, ResourceExt};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde_json::json;

use crate::akapi::{
    auth::get_valid_token,
    user::{Find, FindBody, SetPassword, SetPasswordBody},
    AkApiRoute, AkClient,
};

use super::{crd, labels};

pub async fn reconcile(obj: &crd::AuthentikUser, client: Client) -> Result<()> {
    let instance = obj.spec.authentik_instance.to_string();
    let ns = obj
        .namespace()
        .ok_or(anyhow!("Missing namespace `{}`.", instance.clone()))?;

    // Get the API key.
    let api_key = get_valid_token(client.clone(), &ns, &instance).await?;
    let ak = AkClient::new(&api_key, &instance, &ns)?;

    // Check if the secret already exists.
    let secret_api: Api<Secret> = Api::namespaced(client.clone(), &ns);
    if secret_api
        .get_opt(&format!("ak-{}-user-{}", instance, obj.name_any()))
        .await?
        .is_some()
    {
        return Ok(());
    }

    // Find the user.
    let result = Find::send(
        &ak,
        FindBody {
            username: Some(obj.spec.username.clone()),
            ..Default::default()
        },
    )
    .await?;

    let user = match result
        .iter()
        .find(|&user| user.username == obj.spec.username)
    {
        Some(user) => user,
        None => return Err(anyhow!("Failed to find user `{}`.", obj.spec.username)),
    };

    // Generate a random password.
    let password: String = obj.spec.password.clone().unwrap_or_else(|| {
        thread_rng()
            .sample_iter(&Alphanumeric)
            .take(128)
            .map(char::from)
            .collect()
    });

    // Set the password on the user.
    SetPassword::send(
        &ak,
        SetPasswordBody {
            id: user.pk,
            password: password.clone(),
        },
    )
    .await?;

    // Create the secret.
    secret_api
        .create(
            &PostParams::default(),
            &build(instance.clone(), obj, password)?,
        )
        .await?;

    info!("Set the password for the user `{}`.", obj.name_any());

    Ok(())
}

pub async fn cleanup(_obj: &crd::AuthentikUser, _client: Client) -> Result<()> {
    // Note: The secret will automatically be cleaned up by Kubernetes.
    Ok(())
}

fn build(name: String, obj: &crd::AuthentikUser, password: String) -> Result<Secret> {
    let secret: Secret = serde_json::from_value(json!({
        "apiVersion": "v1",
        "kind": "Secret",
        "type": "Opaque",
        "metadata": {
            "name": format!("ak-{}-user-{}", &name, obj.name_any()),
            "labels": labels::get_labels(name.clone(), "password".to_string()),
            "ownerReferences": [{
                "apiVersion": "ak.dany.dev/v1",
                "kind": "AuthentikUser",
                "name": obj.name_any(),
                "uid": obj.uid().expect("Failed to get UID of Authentik."),
                "controller": true,
            }]
        },
        "stringData": {
            "username": obj.spec.username,
            "email": obj.spec.email,
            "password": password
        }
    }))?;

    Ok(secret)
}
