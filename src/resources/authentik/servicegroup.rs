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

    // Check if the group exists first.
    let groups = FindGroup::send(
        &ak,
        FindGroupBody {
            name: Some(service_group_name(&instance)),
            ..Default::default()
        },
    )
    .await?;

    if !groups.is_empty() {
        return Ok(());
    }

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

    // Create the group.
    let result = CreateGroup::send(
        &ak,
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
            info!("Service group created with ID `{}`.", group.pk);
            Ok(())
        }
        Err(CreateGroupError::ExistsError) => Ok(()),
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

    // Find the group ID.
    let result = FindGroup::send(
        &ak,
        FindGroupBody {
            name: Some(service_group_name(&instance)),
            ..Default::default()
        },
    )
    .await;

    let mut groups = match result {
        Ok(groups) => groups,
        Err(_) => {
            warn!("Failed to get the groups, skipping deleting the service group.");
            return Ok(());
        }
    };

    let group_id = match groups.pop() {
        Some(group) => group.pk,
        None => {
            info!("Group was not found, so cannot delete it.");
            return Ok(());
        }
    };

    // Delete the group.
    match DeleteGroup::send(&ak, group_id).await {
        Ok(_) => {
            info!("Deleted service group.");
        }
        Err(DeleteGroupError::NotFound) => {
            info!("Group was not found, so cannot delete it.");
        }
        Err(e) => {
            warn!(
                "Failed to delete service group, ignoring during deletion. ({})",
                e
            );
        }
    };

    Ok(())
}
