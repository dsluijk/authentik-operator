use anyhow::{anyhow, Result};
use kube::{Client, ResourceExt};

use crate::akapi::{
    auth::get_valid_token,
    flow::{DeleteFlow, DeleteFlowError},
    stages::{DeleteStage, DeleteStageError, FindStage, FindStageBody},
    AkApiRoute, AkServer,
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

    // Create the api and get the key.
    let mut api = AkServer::connect(&instance, &ns, client.clone()).await?;
    let api_key = get_valid_token(&mut api, client.clone(), &ns, &instance).await?;

    // Delete the flow if it exists.
    match DeleteFlow::send(&mut api, &api_key, "initial-setup".to_string()).await {
        Ok(_) => {
            debug!("Initial flow was deleted.");
        }
        Err(DeleteFlowError::NotFound) => {}
        Err(e) => return Err(e.into()),
    }

    // Find and delete the OOBE stages.
    match FindStage::send(
        &mut api,
        &api_key,
        FindStageBody {
            name: Some("default-oobe-password".to_string()),
        },
    )
    .await
    {
        Ok(stages) => {
            if let Some(stage) = stages.first() {
                match DeleteStage::send(&mut api, &api_key, stage.pk.clone()).await {
                    Ok(_) => {
                        debug!("OOBE password stage was deleted.");
                    }
                    Err(DeleteStageError::NotFound) => {}
                    Err(e) => return Err(e.into()),
                }
            }
        }
        Err(e) => return Err(e.into()),
    }

    Ok(())
}

pub async fn cleanup(_obj: &crd::Authentik, _client: Client) -> Result<()> {
    // Note: currently the OOBE is not restored.
    Ok(())
}
