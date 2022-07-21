use anyhow::{anyhow, Result};
use k8s_openapi::api::core::v1::Secret;
use kube::{Api, Client};

use super::{
    user::{GetSelf, GetSelfError},
    AkApiRoute, AkClient,
};

pub static TEMP_AUTH_TOKEN: &str = "AUTHENTIK_TEMP_AUTH_TOKEN";

pub async fn get_valid_token(client: Client, ns: &str, instance: &str) -> Result<String> {
    // Try a token in the secret first.
    if let Some(secret) = get_valid_secret_token(client, ns, instance).await? {
        return Ok(secret);
    }

    // Check if the temporally token is still valid.
    let ak = AkClient::new(TEMP_AUTH_TOKEN, instance, ns)?;
    if validate_token(&ak).await? {
        return Ok(TEMP_AUTH_TOKEN.to_string());
    }

    Err(anyhow!("No valid authentication token was found."))
}

pub async fn get_valid_secret_token(
    client: Client,
    ns: &str,
    instance: &str,
) -> Result<Option<String>> {
    if let Some(secret) = get_token_secret(client, ns, instance).await? {
        let ak = AkClient::new(&secret, instance, ns)?;

        if validate_token(&ak).await? {
            return Ok(Some(secret));
        }
    }

    Ok(None)
}

async fn validate_token(ak: &AkClient) -> Result<bool> {
    match GetSelf::send(ak, ()).await {
        Ok(_) => Ok(true),
        Err(GetSelfError::Forbidden) => Ok(false),
        Err(e) => Err(e.into()),
    }
}

async fn get_token_secret(client: Client, ns: &str, instance: &str) -> Result<Option<String>> {
    // Get the token from a secret, iff it exists.
    let api: Api<Secret> = Api::namespaced(client, ns);
    let name = format!("ak-{}-api-operatortoken", instance);

    if let Some(secret) = api.get_opt(&name).await? {
        let mut data = secret
            .data
            .ok_or(anyhow!("Token secret does not contain any data"))?;
        let token = data
            .remove("token")
            .ok_or(anyhow!("Token secret does not contain a token."))?;
        let token_string = String::from_utf8(token.0)?;

        Ok(Some(token_string))
    } else {
        Ok(None)
    }
}
