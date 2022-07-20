use async_trait::async_trait;
use hyper::{Method, StatusCode};
use serde::Serialize;
use thiserror::Error;

use crate::{
    akapi::{types::Provider, AkApiRoute, AkServer},
    error::AKApiError,
    resources::authentik_provider_oauth::crd,
};

pub struct CreateOAuthProvider;

#[async_trait]
impl AkApiRoute for CreateOAuthProvider {
    type Body = CreateOAuthProviderBody;
    type Response = Provider;
    type Error = CreateOAuthProviderError;

    #[instrument]
    async fn send(
        api: &mut AkServer,
        api_key: &str,
        body: Self::Body,
    ) -> Result<Self::Response, Self::Error> {
        let res = api
            .send(Method::POST, "/api/v3/providers/oauth2/", api_key, body)
            .await?;

        match res.status() {
            StatusCode::CREATED => {
                let bytes = hyper::body::to_bytes(res.into_body())
                    .await
                    .map_err(AKApiError::StreamError)?;
                let body: Provider =
                    serde_json::from_slice(&bytes).map_err(AKApiError::SerializeError)?;

                Ok(body)
            }
            code => Err(Self::Error::Unknown(format!(
                "Invalid status code {}",
                code
            ))),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CreateOAuthProviderBody {
    pub name: String,
    pub authorization_flow: String,
    pub property_mappings: Vec<String>,
    pub client_type: crd::ClientType,
    pub access_code_validity: String,
    pub token_validity: String,
    pub include_claims_in_id_token: bool,
    pub signing_key: Option<String>,
    pub redirect_uris: String,
    pub sub_mode: crd::SubjectMode,
    pub issuer_mode: crd::IssuerMode,
}

#[derive(Error, Debug)]
pub enum CreateOAuthProviderError {
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error(transparent)]
    RequestError(#[from] AKApiError),
}
