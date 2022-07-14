use anyhow::{anyhow, Result};
use kube::{Client, ResourceExt};

use crate::akapi::{
    token::{DeleteToken, DeleteTokenError},
    user::{
        CreateServiceAccount, CreateServiceAccountBody, CreateServiceAccountError, DeleteAccount,
        DeleteAccountError, Find, FindBody,
    },
    AkApiRoute, AkServer, API_USER, TEMP_AUTH_TOKEN,
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

    // Attempt to create the account.
    let mut api = AkServer::connect(&instance, &ns, client.clone()).await?;
    let result = CreateServiceAccount::send(
        &mut api,
        TEMP_AUTH_TOKEN,
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
        Err(CreateServiceAccountError::ExistsError) => {
            debug!("Service account already exists, assuming it's correct.");
        }
        Err(e) => return Err(e.into()),
    };

    // Delete the password token for this account if it exists.
    let result = DeleteToken::send(
        &mut api,
        TEMP_AUTH_TOKEN,
        format!("service-account-{}-password", API_USER),
    )
    .await;

    match result {
        Ok(_) => {
            debug!("Service account password deleted.");
            Ok(())
        }
        Err(DeleteTokenError::NotFound) => {
            debug!("The password does not exist, continuing.");
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

pub async fn cleanup(obj: &crd::Authentik, client: Client) -> Result<()> {
    // TODO: discover working auth token.
    let instance = obj
        .metadata
        .name
        .clone()
        .ok_or(anyhow!("Missing instance name.".to_string()))?;
    let ns = obj
        .namespace()
        .ok_or(anyhow!("Missing namespace `{}`.", instance.clone()))?;

    let mut api = AkServer::connect(&instance, &ns, client.clone()).await?;
    let mut result = Find::send(
        &mut api,
        TEMP_AUTH_TOKEN,
        FindBody {
            username: Some(API_USER.to_string()),
            ..Default::default()
        },
    )
    .await?;

    let user = match result.pop() {
        Some(user) => user,
        None => {
            debug!("Operator user does not exist, skipping deleting it.");
            return Ok(());
        }
    };

    match DeleteAccount::send(&mut api, TEMP_AUTH_TOKEN, user.pk).await {
        Ok(_) => {
            debug!("Deleted operator user.");
            Ok(())
        }
        Err(DeleteAccountError::NotFound) => {
            debug!("User was not found for deletion.");
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}
