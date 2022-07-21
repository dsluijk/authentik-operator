use anyhow::{anyhow, Result};
use kube::{Client, ResourceExt};

use crate::akapi::{
    auth::get_valid_token,
    flow::{DeleteFlow, DeleteFlowError},
    stages::{DeleteStage, DeleteStageError, FindStage, FindStageBody},
    AkApiRoute, AkClient,
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
    let api_key = get_valid_token(client.clone(), &ns, &instance).await?;
    let ak = AkClient::new(&api_key, &instance, &ns)?;

    // Delete the flow if it exists.
    match DeleteFlow::send(&ak, "initial-setup".to_string()).await {
        Ok(_) => {
            debug!("Initial flow was deleted.");
        }
        Err(DeleteFlowError::NotFound) => {}
        Err(e) => return Err(e.into()),
    }

    // Find and delete the OOBE stages.
    match FindStage::send(
        &ak,
        FindStageBody {
            name: Some("default-oobe-password".to_string()),
        },
    )
    .await
    {
        Ok(stages) => {
            if let Some(stage) = stages.first() {
                match DeleteStage::send(&ak, stage.pk.clone()).await {
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
