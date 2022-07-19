use anyhow::{anyhow, Result};
use kube::{Client, ResourceExt};

use crate::akapi::{
    auth::get_valid_token,
    group::{FindGroup, FindGroupBody},
    user::{Find, FindBody, UpdateUser, UpdateUserBody},
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

    // Find the ID's of the groups.
    let mut group_ids = Vec::new();

    for group_name in obj.spec.groups.as_ref().unwrap_or(&Vec::new()) {
        let result = FindGroup::send(
            &mut api,
            &api_key,
            FindGroupBody {
                name: Some(group_name.to_string()),
                ..Default::default()
            },
        )
        .await?;

        let group = match result.iter().find(|&group| &group.name == group_name) {
            Some(group) => group,
            None => return Err(anyhow!("Failed to find group `{}`", group_name)),
        };

        group_ids.push(group.pk.clone());
    }

    // Find the user.
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
        None => return Err(anyhow!("Failed to find user `{}`.", obj.spec.username)),
    };

    // Update the user groups.
    UpdateUser::send(
        &mut api,
        &api_key,
        UpdateUserBody {
            id: user.pk,
            groups: Some(group_ids),
            ..Default::default()
        },
    )
    .await?;

    Ok(())
}

pub async fn cleanup(_obj: &crd::AuthentikUser, _client: Client) -> Result<()> {
    Ok(())
}
