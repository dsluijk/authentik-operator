use anyhow::{anyhow, Result};
use kube::{Client, ResourceExt};

use crate::akapi::{
    auth::get_valid_token,
    group::{
        CreateGroup, CreateGroupBody, CreateGroupError, DeleteGroup, DeleteGroupError, FindGroup,
        FindGroupBody,
    },
    AkApiRoute, AkServer,
};

use super::crd;

pub async fn reconcile(obj: &crd::AuthentikGroup, client: Client) -> Result<()> {
    let instance = obj.spec.authentik_instance.to_string();
    let ns = obj
        .namespace()
        .ok_or(anyhow!("Missing namespace `{}`.", instance.clone()))?;

    // Get the API key.
    let mut api = AkServer::connect(&instance, &ns, client.clone()).await?;
    let api_key = get_valid_token(&mut api, client.clone(), &ns, &instance).await?;

    // Check if the group already exists.
    let result = FindGroup::send(
        &mut api,
        &api_key,
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
            &mut api,
            &api_key,
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
        &mut api,
        &api_key,
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
    let mut api = AkServer::connect(&instance, &ns, client.clone()).await?;
    let api_key = get_valid_token(&mut api, client.clone(), &ns, &instance).await?;

    // Find the ID of the group to delete.
    let result = FindGroup::send(
        &mut api,
        &api_key,
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
    match DeleteGroup::send(&mut api, &api_key, group_id).await {
        Ok(_) => {
            debug!("Deleted service group.");
            Ok(())
        }
        Err(DeleteGroupError::NotFound) => Ok(()),
        Err(e) => Err(e.into()),
    }
}
