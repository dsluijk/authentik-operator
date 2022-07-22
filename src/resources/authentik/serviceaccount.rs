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
    AkApiRoute, AkClient, API_USER,
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
    let api_key = get_valid_token(client.clone(), &ns, &instance).await?;
    let ak = AkClient::new(&api_key, &instance, &ns)?;

    // Attempt to create the account.
    let result = CreateServiceAccount::send(
        &ak,
        CreateServiceAccountBody {
            name: API_USER.to_string(),
            create_group: false,
        },
    )
    .await;

    match result {
        Ok(account) => {
            info!("Service account created with ID `{}`.", account.user_uid);
        }
        Err(CreateServiceAccountError::ExistsError) => {}
        Err(e) => return Err(e.into()),
    };

    // Delete the password token for this account if it exists.
    let result = DeleteToken::send(&ak, format!("service-account-{}-password", API_USER)).await;

    match result {
        Ok(_) => {
            info!("Service account password deleted.");
        }
        Err(DeleteTokenError::NotFound) => {}
        Err(e) => return Err(e.into()),
    };

    // Get the ID of the service account.
    let mut users = Find::send(
        &ak,
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
        &ak,
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
            info!("Token for the service account was created.");
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
    let api_key = get_valid_token(client.clone(), &ns, &instance).await?;
    let ak = AkClient::new(&api_key, &instance, &ns)?;

    let result = Find::send(
        &ak,
        FindBody {
            username: Some(API_USER.to_string()),
            ..Default::default()
        },
    )
    .await;

    let users = match result {
        Ok(users) => users,
        Err(_) => {
            warn!("Failed to get the users, skipping deleting the operator user.");
            return Ok(());
        }
    };

    let user = match users.iter().find(|&user| &user.username == API_USER) {
        Some(user) => user,
        None => {
            return Ok(());
        }
    };

    match DeleteAccount::send(&ak, user.pk).await {
        Ok(_) => {
            info!("Deleted operator user.");
        }
        Err(DeleteAccountError::NotFound) => {}
        Err(e) => {
            warn!(
                "Failed to delete the operator user. Ignoring this during uninstall ({}).",
                e
            );
        }
    }

    Ok(())
}
