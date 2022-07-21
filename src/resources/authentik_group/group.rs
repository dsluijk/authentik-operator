use anyhow::{anyhow, Result};
use kube::{Client, ResourceExt};

use crate::akapi::{
    auth::get_valid_token,
    group::{
        CreateGroup, CreateGroupBody, CreateGroupError, DeleteGroup, DeleteGroupError, FindGroup,
        FindGroupBody,
    },
    AkApiRoute, AkClient,
};

use super::crd;

pub async fn reconcile(obj: &crd::AuthentikGroup, client: Client) -> Result<()> {
    let instance = obj.spec.authentik_instance.to_string();
    let ns = obj
        .namespace()
        .ok_or(anyhow!("Missing namespace `{}`.", instance.clone()))?;

    // Get the API key.
    let api_key = get_valid_token(client.clone(), &ns, &instance).await?;
    let ak = AkClient::new(&api_key, &instance, &ns)?;

    // Check if the group already exists.
    let result = FindGroup::send(
        &ak,
        FindGroupBody {
            name: Some(obj.spec.name.clone()),
            ..Default::default()
        },
    )
    .await?;

    match result.iter().find(|&group| group.name == obj.spec.name) {
        Some(_) => return Ok(()),
        None => (),
    }

    // Get the ID of the parent.
    let parent = if let Some(parent) = obj.spec.parent.clone() {
        let result = FindGroup::send(
            &ak,
            FindGroupBody {
                name: Some(parent.clone()),
                ..Default::default()
            },
        )
        .await?;

        match result.iter().find(|&group| group.name == parent) {
            Some(group) => group.pk.clone(),
            None => return Err(anyhow!("Cannot find parent group `{}`.", parent)),
        }
    } else {
        "".to_string()
    };

    // Try to create the group.
    let result = CreateGroup::send(
        &ak,
        CreateGroupBody {
            name: obj.spec.name.clone(),
            is_superuser: obj.spec.superuser.unwrap_or(false),
            users: Vec::new(),
            parent,
        },
    )
    .await;

    match result {
        Ok(_) => Ok(()),
        Err(CreateGroupError::ExistsError) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

pub async fn cleanup(obj: &crd::AuthentikGroup, client: Client) -> Result<()> {
    let instance = obj.spec.authentik_instance.to_string();
    let ns = obj
        .namespace()
        .ok_or(anyhow!("Missing namespace `{}`.", instance.clone()))?;

    // Get the API key.
    let api_key = get_valid_token(client.clone(), &ns, &instance).await?;
    let ak = AkClient::new(&api_key, &instance, &ns)?;

    // Find the ID of the group to delete.
    let result = FindGroup::send(
        &ak,
        FindGroupBody {
            name: Some(obj.spec.name.clone()),
            ..Default::default()
        },
    )
    .await?;

    let group_id = match result.iter().find(|&group| group.name == obj.spec.name) {
        Some(group) => group.pk.clone(),
        None => return Ok(()),
    };

    // Delete the group.
    match DeleteGroup::send(&ak, group_id).await {
        Ok(_) => {
            debug!("Deleted service group.");
            Ok(())
        }
        Err(DeleteGroupError::NotFound) => Ok(()),
        Err(e) => Err(e.into()),
    }
}
