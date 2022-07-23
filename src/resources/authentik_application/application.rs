use anyhow::{anyhow, Result};
use kube::{Client, ResourceExt};

use crate::akapi::{
    application::{
        CreateApplication, DeleteApplication, DeleteApplicationError, GetApplication,
        PatchApplication,
    },
    auth::get_valid_token,
    provider::{FindProvider, FindProviderBody},
    types::{Application, Provider},
    AkApiRoute, AkClient,
};

use super::crd;

pub async fn reconcile(obj: &crd::AuthentikApplication, client: Client) -> Result<()> {
    let instance = obj.spec.authentik_instance.to_string();
    let ns = obj
        .namespace()
        .ok_or(anyhow!("Missing namespace `{}`.", instance.clone()))?;

    // Get the API key.
    let api_key = get_valid_token(client.clone(), &ns, &instance).await?;
    let ak = AkClient::new(&api_key, &instance, &ns)?;

    // Get the provider, returns an error if it can't be found.
    let providers = FindProvider::send(
        &ak,
        FindProviderBody {
            search: Some(obj.spec.provider.clone()),
        },
    )
    .await?;

    let provider = providers
        .iter()
        .find(|&provider| provider.name == obj.spec.provider)
        .ok_or(anyhow!(
            "Provider `{}` was not found for application `{}`.",
            obj.spec.provider,
            obj.name_any()
        ))?;

    // Get the application, create or patch depending on if it exists.
    match GetApplication::send(&ak, obj.spec.slug.clone()).await? {
        Some(app) => {
            let new_app = build_application(obj.spec.clone(), &provider);
            // Compare the serialized versions of the applications.
            // The non-serialized object contains values we don't care about, and can conflict.
            if serde_json::to_string(&app)? != serde_json::to_string(&new_app)? {
                // There is a difference in the objects, patching it.
                PatchApplication::send(&ak, new_app).await?;
            }
        }
        None => {
            let body = build_application(obj.spec.clone(), &provider);
            CreateApplication::send(&ak, body).await?;
        }
    };

    Ok(())
}

pub async fn cleanup(obj: &crd::AuthentikApplication, client: Client) -> Result<()> {
    let instance = obj.spec.authentik_instance.to_string();
    let ns = obj
        .namespace()
        .ok_or(anyhow!("Missing namespace `{}`.", instance.clone()))?;

    // Get the API key.
    let api_key = get_valid_token(client.clone(), &ns, &instance).await?;
    let ak = AkClient::new(&api_key, &instance, &ns)?;

    // Delete the application.
    match DeleteApplication::send(&ak, obj.spec.name.clone()).await {
        Ok(_) => {
            info!("Deleted application `{}`.", obj.spec.name);
        }
        Err(DeleteApplicationError::NotFound) => {}
        Err(e) => return Err(e.into()),
    };

    Ok(())
}

fn build_application(spec: crd::AuthentikApplicationSpec, provider: &Provider) -> Application {
    Application {
        pk: "".to_string(),
        name: spec.name,
        slug: spec.slug,
        provider: Some(provider.pk),
        provider_obj: None,
        policy_engine_mode: Some(spec.policy_mode),
        group: spec.group,
        open_in_new_tab: Some(spec.ui.new_tab),
        meta_launch_url: spec.ui.url,
        meta_description: Some(spec.ui.description),
        meta_publisher: Some(spec.ui.publisher),
    }
}
