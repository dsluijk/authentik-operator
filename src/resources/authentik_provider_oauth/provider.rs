use anyhow::{anyhow, Result};
use kube::{Client, ResourceExt};

use crate::akapi::{
    auth::get_valid_token,
    certificate::{FindCertificate, FindCertificateBody},
    flow::GetFlow,
    propertymappings::{FindScopeMapping, FindScopeMappingBody},
    provider::{
        CreateOAuthProvider, CreateOAuthProviderBody, DeleteOAuthProvider,
        DeleteOAuthProviderError, FindOAuthProvider, FindOAuthProviderBody,
    },
    AkApiRoute, AkClient,
};

use super::crd;

pub async fn reconcile(obj: &crd::AuthentikOAuthProvider, client: Client) -> Result<()> {
    let instance = obj.spec.authentik_instance.to_string();
    let ns = obj
        .namespace()
        .ok_or(anyhow!("Missing namespace `{}`.", instance.clone()))?;

    // Get the API key.
    let api_key = get_valid_token(client.clone(), &ns, &instance).await?;
    let ak = AkClient::new(&api_key, &instance, &ns)?;

    // Check if the provider already exists.
    let providers = FindOAuthProvider::send(
        &ak,
        FindOAuthProviderBody {
            name: Some(obj.spec.name.clone()),
        },
    )
    .await?;

    match providers
        .iter()
        .find(|&provider| provider.name == obj.spec.name)
    {
        Some(_) => return Ok(()),
        None => (),
    }

    // Get the flow.
    let flow = GetFlow::send(&ak, obj.spec.flow.clone()).await?;

    // Get the ID's of the scopes.
    let mut scopes = Vec::new();
    for scope in &obj.spec.scopes {
        let mappings = FindScopeMapping::send(
            &ak,
            FindScopeMappingBody {
                name: Some(scope.clone()),
            },
        )
        .await?;

        let mapping = match mappings.iter().find(|&mapping| &mapping.name == scope) {
            Some(mapping) => mapping,
            None => return Err(anyhow!("Cannot find scope `{}`.", scope)),
        };

        scopes.push(mapping.pk.clone());
    }

    // Get the ID of the signing key.
    let signing_key = if let Some(signing_key) = obj.spec.signing_key.clone() {
        let certificates = FindCertificate::send(
            &ak,
            FindCertificateBody {
                name: Some(signing_key.clone()),
                has_keys: Some(true),
            },
        )
        .await?;

        let certificate = match certificates
            .iter()
            .find(|&certificate| certificate.name == signing_key)
        {
            Some(certificate) => certificate,
            None => return Err(anyhow!("Cannot find signing key `{}`.", signing_key)),
        };

        Some(certificate.pk.clone())
    } else {
        None
    };

    // Create the provider.
    CreateOAuthProvider::send(
        &ak,
        CreateOAuthProviderBody {
            signing_key,
            name: obj.spec.name.clone(),
            authorization_flow: flow.pk,
            property_mappings: scopes,
            client_type: obj.spec.client_type.clone(),
            include_claims_in_id_token: obj.spec.claims_in_token,
            redirect_uris: obj.spec.redirect_uris.join("\n"),
            access_code_validity: obj.spec.access_code_validity.clone(),
            token_validity: obj.spec.token_validity.clone(),
            sub_mode: obj.spec.subject_mode.clone(),
            issuer_mode: obj.spec.issuer_mode.clone(),
        },
    )
    .await?;

    Ok(())
}

pub async fn cleanup(obj: &crd::AuthentikOAuthProvider, client: Client) -> Result<()> {
    let instance = obj.spec.authentik_instance.to_string();
    let ns = obj
        .namespace()
        .ok_or(anyhow!("Missing namespace `{}`.", instance.clone()))?;

    // Get the API key.
    let api_key = get_valid_token(client.clone(), &ns, &instance).await?;
    let ak = AkClient::new(&api_key, &instance, &ns)?;

    // Find the provider.
    let providers = FindOAuthProvider::send(
        &ak,
        FindOAuthProviderBody {
            name: Some(obj.spec.name.clone()),
        },
    )
    .await?;

    let provider = match providers
        .iter()
        .find(|&provider| provider.name == obj.spec.name)
    {
        Some(provider) => provider,
        None => return Ok(()),
    };

    // Delete the provider.
    match DeleteOAuthProvider::send(&ak, provider.pk.clone()).await {
        Ok(_) => {
            info!("OAuth provider `{}` was deleted.", obj.name_any());
            Ok(())
        }
        Err(DeleteOAuthProviderError::NotFound) => Ok(()),
        Err(e) => Err(e.into()),
    }
}
