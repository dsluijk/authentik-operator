use anyhow::{anyhow, Result};
use k8s_openapi::api::core::v1::Secret;
use kube::{Api, Client};

use super::{
    user::{GetSelf, GetSelfError},
    AkApiRoute, AkServer,
};

pub static TEMP_AUTH_TOKEN: &str = "AUTHENTIK_TEMP_AUTH_TOKEN";

pub async fn get_valid_token(
    api: &mut AkServer,
    client: Client,
    ns: &str,
    instance: &str,
) -> Result<String> {
    // Try a token in the secret first.
    if let Some(secret) = get_token_secret(client, ns, instance).await? {
        if validate_token(api, &secret).await? {
            return Ok(secret);
        }
    }

    // Check if the temporally token is still valid.
    if validate_token(api, TEMP_AUTH_TOKEN).await? {
        return Ok(TEMP_AUTH_TOKEN.to_string());
    }

    Err(anyhow!("No valid authentication token was found."))
}

async fn validate_token(api: &mut AkServer, token: &str) -> Result<bool> {
    match GetSelf::send(api, token, ()).await {
        Ok(_) => Ok(true),
        Err(GetSelfError::Forbidden) => Ok(false),
        Err(e) => Err(e.into()),
    }
}

async fn get_token_secret(client: Client, ns: &str, instance: &str) -> Result<Option<String>> {
    // Create or patch the secret.
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
