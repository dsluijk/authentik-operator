use anyhow::{anyhow, Result};
use kube::{Client, ResourceExt};

use crate::akapi::{
    auth::get_valid_token,
    group::{
        CreateGroup, CreateGroupBody, CreateGroupError, DeleteGroup, DeleteGroupError, FindGroup,
        FindGroupBody,
    },
    service_group_name,
    user::{Find, FindBody},
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

    // Check if the group exists first.
    let groups = FindGroup::send(
        &mut api,
        &api_key,
        FindGroupBody {
            name: Some(service_group_name(&instance)),
            ..Default::default()
        },
    )
    .await?;

    if !groups.is_empty() {
        debug!("Service group seems to exist, skipping creation");
        return Ok(());
    }

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

    // Create the group.
    let result = CreateGroup::send(
        &mut api,
        &api_key,
        CreateGroupBody {
            name: service_group_name(&instance),
            is_superuser: true,
            parent: "".to_string(),
            users: vec![user_id],
        },
    )
    .await;

    match result {
        Ok(group) => {
            debug!("Service group created with ID `{}`.", group.pk);
            Ok(())
        }
        Err(CreateGroupError::ExistsError) => {
            debug!("Service account already exists, assuming that's correct.");
            Ok(())
        }
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

    // Find the group ID.
    let mut groups = FindGroup::send(
        &mut api,
        &api_key,
        FindGroupBody {
            name: Some(service_group_name(&instance)),
            ..Default::default()
        },
    )
    .await?;

    let group_id = match groups.pop() {
        Some(group) => group.pk,
        None => {
            debug!("Group was not found, so cannot delete it.");
            return Ok(());
        }
    };

    // Delete the group.
    match DeleteGroup::send(&mut api, &api_key, group_id).await {
        Ok(_) => {
            debug!("Deleted service group.");
            Ok(())
        }
        Err(DeleteGroupError::NotFound) => {
            debug!("Group was not found, so cannot delete it.");
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}
