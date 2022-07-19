use anyhow::{anyhow, Result};
use kube::{Client, ResourceExt};

use crate::akapi::{
    auth::get_valid_token,
    user::{CreateAccount, CreateAccountBody, DeleteAccount, DeleteAccountError, Find, FindBody},
    AkApiRoute, AkServer,
};

use super::crd;

pub async fn reconcile(obj: &crd::AuthentikUser, client: Client) -> Result<()> {
    let instance = obj.spec.authentik_instance.to_string();
    let ns = obj
        .namespace()
        .ok_or(anyhow!("Missing namespace `{}`.", instance.clone()))?;

    // Get the API key.
    let mut api = AkServer::connect(&instance, &ns, client.clone()).await?;
    let api_key = get_valid_token(&mut api, client.clone(), &ns, &instance).await?;

    // Check if the account already exists.
    let result = Find::send(
        &mut api,
        &api_key,
        FindBody {
            username: Some(obj.spec.username.clone()),
            ..Default::default()
        },
    )
    .await?;

    match result
        .iter()
        .find(|&user| user.username == obj.spec.username)
    {
        Some(_user) => return Ok(()),
        None => (),
    };

    // Create the account as it does not exists.
    CreateAccount::send(
        &mut api,
        &api_key,
        CreateAccountBody {
            name: obj.spec.display_name.clone(),
            username: obj.spec.username.clone(),
            email: obj.spec.email.clone(),
            path: obj.spec.path.clone(),
            groups: Vec::new(),
        },
    )
    .await?;

    Ok(())
}

pub async fn cleanup(obj: &crd::AuthentikUser, client: Client) -> Result<()> {
    let instance = obj.spec.authentik_instance.to_string();
    let ns = obj
        .namespace()
        .ok_or(anyhow!("Missing namespace `{}`.", instance.clone()))?;

    // Get the API key.
    let mut api = AkServer::connect(&instance, &ns, client.clone()).await?;
    let api_key = get_valid_token(&mut api, client.clone(), &ns, &instance).await?;

    let result = Find::send(
        &mut api,
        &api_key,
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
        None => return Ok(()),
    };

    match DeleteAccount::send(&mut api, &api_key, user.pk).await {
        Ok(_) => {
            debug!("Deleted user {}.", obj.spec.username);
            Ok(())
        }
        Err(DeleteAccountError::NotFound) => Ok(()),
        Err(e) => Err(e.into()),
    }
}
