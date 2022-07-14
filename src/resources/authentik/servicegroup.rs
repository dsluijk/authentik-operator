use anyhow::{anyhow, Result};
use kube::{Client, ResourceExt};

use crate::akapi::{
    group::{
        CreateGroup, CreateGroupBody, CreateGroupError, DeleteGroup, DeleteGroupError, FindGroup,
        FindGroupBody,
    },
    service_group_name,
    user::{Find, FindBody},
    AkApiRoute, AkServer, API_USER, TEMP_AUTH_TOKEN,
};

use super::crd;

pub async fn reconcile(obj: &crd::Authentik, client: Client) -> Result<()> {
    let instance = obj
        .metadata
        .name
        .clone()
        .ok_or(anyhow!("Missing instance name.".to_string(),))?;
    let ns = obj
        .namespace()
        .ok_or(anyhow!("Missing namespace `{}`.", instance.clone()))?;

    // Check if the group already exists.
    let api = AkServer::connect(&instance, &ns, client.clone()).await?;
    let groups = FindGroup::send(
        api,
        TEMP_AUTH_TOKEN,
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
    let api = AkServer::connect(&instance, &ns, client.clone()).await?;
    let mut users = Find::send(
        api,
        TEMP_AUTH_TOKEN,
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
    let api = AkServer::connect(&instance, &ns, client).await?;
    let result = CreateGroup::send(
        api,
        TEMP_AUTH_TOKEN,
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
        .ok_or(anyhow!("Missing instance name.".to_string(),))?;
    let ns = obj
        .namespace()
        .ok_or(anyhow!("Missing namespace `{}`.", instance.clone()))?;

    // Find the group ID.
    let api = AkServer::connect(&instance, &ns, client.clone()).await?;
    let mut groups = FindGroup::send(
        api,
        TEMP_AUTH_TOKEN,
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
    let api = AkServer::connect(&instance, &ns, client.clone()).await?;
    match DeleteGroup::send(api, TEMP_AUTH_TOKEN, group_id).await {
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
