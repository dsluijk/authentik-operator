use anyhow::{anyhow, Result};
use kube::{Client, ResourceExt};

use crate::akapi::{
    auth::get_valid_token,
    token::{CreateToken, CreateTokenBody, CreateTokenError, DeleteToken, DeleteTokenError},
    token_identifier_name,
    user::{
        CreateServiceAccount, CreateServiceAccountBody, CreateServiceAccountError, DeleteAccount,
        DeleteAccountError, Find, FindBody,
    },
    AkApiRoute, AkServer, API_USER,
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

    // Get the API key.
    let mut api = AkServer::connect(&instance, &ns, client.clone()).await?;
    let api_key = get_valid_token(&mut api, client.clone(), &ns, &instance).await?;

    // Attempt to create the account.
    let result = CreateServiceAccount::send(
        &mut api,
        &api_key,
        CreateServiceAccountBody {
            name: API_USER.to_string(),
            create_group: false,
        },
    )
    .await;

    match result {
        Ok(account) => {
            debug!("Service account created with ID `{}`.", account.user_uid);
        }
        Err(CreateServiceAccountError::ExistsError) => {}
        Err(e) => return Err(e.into()),
    };

    // Delete the password token for this account if it exists.
    let result = DeleteToken::send(
        &mut api,
        &api_key,
        format!("service-account-{}-password", API_USER),
    )
    .await;

    match result {
        Ok(_) => {
            debug!("Service account password deleted.");
        }
        Err(DeleteTokenError::NotFound) => {}
        Err(e) => return Err(e.into()),
    };

    // Get the ID of the service account.
    let mut users = Find::send(
        &mut api,
        &api_key,
        FindBody {
            username: Some(API_USER.to_string()),
            ..Default::default()
        },
    )
    .await?;

    let user_id = match users.pop() {
        Some(user) => user.pk,
        None => {
            return Err(anyhow!("The server account was not found!"));
        }
    };

    // Create the api token if it does not exist.
    let result = CreateToken::send(
        &mut api,
        &api_key,
        CreateTokenBody {
            identifier: token_identifier_name(&instance, "operatortoken"),
            intent: "api".to_string(),
            user: user_id,
            description: "Authentication token for the Authentik Operator. Do not delete!"
                .to_string(),
            expiring: false,
        },
    )
    .await;

    match result {
        Ok(_) => {
            debug!("Token for the service account was created.");
            Ok(())
        }
        Err(CreateTokenError::ExistsError) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

pub async fn cleanup(obj: &crd::Authentik, client: Client) -> Result<()> {
    let instance = obj
        .metadata
        .name
        .clone()
        .ok_or(anyhow!("Missing instance name.".to_string()))?;
    let ns = obj
        .namespace()
        .ok_or(anyhow!("Missing namespace `{}`.", instance.clone()))?;

    // Get the API key.
    let mut api = AkServer::connect(&instance, &ns, client.clone()).await?;
    let api_key = get_valid_token(&mut api, client.clone(), &ns, &instance).await?;

    let mut result = Find::send(
        &mut api,
        &api_key,
        FindBody {
            username: Some(API_USER.to_string()),
            ..Default::default()
        },
    )
    .await?;

    let user = match result.pop() {
        Some(user) => user,
        None => {
            return Ok(());
        }
    };

    match DeleteAccount::send(&mut api, &api_key, user.pk).await {
        Ok(_) => {
            debug!("Deleted operator user.");
            Ok(())
        }
        Err(DeleteAccountError::NotFound) => Ok(()),
        Err(e) => Err(e.into()),
    }
}
