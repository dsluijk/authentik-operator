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
    AkApiRoute, AkServer,
};

use super::crd::{self, IssuerMode, SubjectMode};

pub async fn reconcile(obj: &crd::AuthentikOAuthProvider, client: Client) -> Result<()> {
    let instance = obj.spec.authentik_instance.to_string();
    let ns = obj
        .namespace()
        .ok_or(anyhow!("Missing namespace `{}`.", instance.clone()))?;

    // Get the API key.
    let mut api = AkServer::connect(&instance, &ns, client.clone()).await?;
    let api_key = get_valid_token(&mut api, client.clone(), &ns, &instance).await?;

    // Check if the provider already exists.
    let providers = FindOAuthProvider::send(
        &mut api,
        &api_key,
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
    let flow = GetFlow::send(&mut api, &api_key, obj.spec.flow.clone()).await?;

    // Get the ID's of the scopes.
    let mut scopes = Vec::new();
    for scope in &obj.spec.scopes {
        let mappings = FindScopeMapping::send(
            &mut api,
            &api_key,
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
            &mut api,
            &api_key,
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
    let access_code_validity = obj
        .spec
        .access_code_validity
        .clone()
        .unwrap_or("minutes=1".to_string());
    let token_validity = obj
        .spec
        .token_validity
        .clone()
        .unwrap_or("days=30".to_string());
    let sub_mode = obj
        .spec
        .subject_mode
        .clone()
        .unwrap_or(SubjectMode::HashedUserId);
    let issuer_mode = obj
        .spec
        .issuer_mode
        .clone()
        .unwrap_or(IssuerMode::PerProvider);

    CreateOAuthProvider::send(
        &mut api,
        &api_key,
        CreateOAuthProviderBody {
            name: obj.spec.name.clone(),
            authorization_flow: flow.pk,
            property_mappings: scopes,
            client_type: obj.spec.client_type.clone(),
            include_claims_in_id_token: obj.spec.claims_in_token.unwrap_or(true),
            redirect_uris: obj.spec.redirect_uris.join("\n"),
            access_code_validity,
            token_validity,
            signing_key,
            sub_mode,
            issuer_mode,
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
    let mut api = AkServer::connect(&instance, &ns, client.clone()).await?;
    let api_key = get_valid_token(&mut api, client.clone(), &ns, &instance).await?;

    // Find the provider.
    let providers = FindOAuthProvider::send(
        &mut api,
        &api_key,
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
    match DeleteOAuthProvider::send(&mut api, &api_key, provider.pk.clone()).await {
        Ok(_) => {
            debug!("OAuth provider `{}` was deleted.", obj.name_any());
            Ok(())
        }
        Err(DeleteOAuthProviderError::NotFound) => Ok(()),
        Err(e) => Err(e.into()),
    }
}
