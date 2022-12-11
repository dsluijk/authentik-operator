use std::collections::{hash_map::RandomState, HashSet};

use anyhow::{anyhow, Result};
use kube::{Client, ResourceExt};

use crate::akapi::{
    auth::get_valid_token,
    certificate::{FindCertificate, FindCertificateBody},
    flow::GetFlow,
    propertymappings::{FindScopeMapping, FindScopeMappingBody},
    provider::{
        CreateOAuthProvider, DeleteOAuthProvider, DeleteOAuthProviderError, FindOAuthProvider,
        FindOAuthProviderBody, PatchOAuthProvider,
    },
    types::{Flow, OAuthProvider},
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

    // Check if the provider already exists.
    let providers = FindOAuthProvider::send(
        &ak,
        FindOAuthProviderBody {
            name: Some(obj.spec.name.clone()),
        },
    )
    .await?;
    let provider = providers
        .iter()
        .find(|&provider| provider.name == obj.spec.name);

    let new_provider = build_provider(&obj.spec, provider, &flow, signing_key, scopes);
    match provider {
        Some(provider) => {
            // Compare the serialized versions of the provider.
            // The non-serialized object contains values we don't care about, and can conflict.
            if serde_json::to_string(&provider)? != serde_json::to_string(&new_provider)? {
                // There is a difference in the objects, patching it.
                PatchOAuthProvider::send(&ak, new_provider).await?;
            }
        }
        None => {
            // Create the provider.
            CreateOAuthProvider::send(&ak, new_provider).await?;
        }
    }

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

fn build_provider(
    spec: &crd::AuthentikOAuthProviderSpec,
    old_provider: Option<&OAuthProvider>,
    flow: &Flow,
    signing_key: Option<String>,
    scopes: Vec<String>,
) -> OAuthProvider {
    let mappings = old_provider
        .and_then(|p| p.property_mappings.clone())
        .filter(|mappings| {
            let old_set: HashSet<String, RandomState> = HashSet::from_iter(mappings.to_owned());
            let new_set: HashSet<String, RandomState> = HashSet::from_iter(scopes.clone());

            old_set == new_set
        })
        .unwrap_or(scopes);

    OAuthProvider {
        pk: old_provider.map(|p| p.pk).unwrap_or(0),
        signing_key,
        name: spec.name.clone(),
        authorization_flow: flow.pk.clone(),
        property_mappings: Some(mappings),
        client_type: Some(spec.client_type.clone()),
        client_id: Some(spec.client_id.clone()),
        client_secret: Some(spec.client_secret.clone()),
        include_claims_in_id_token: spec.claims_in_token,
        redirect_uris: Some(spec.redirect_uris.join("\n")),
        access_code_validity: Some(spec.access_code_validity.clone()),
        token_validity: Some(spec.token_validity.clone()),
        sub_mode: Some(spec.subject_mode.clone()),
        issuer_mode: Some(spec.issuer_mode.clone()),
    }
}
