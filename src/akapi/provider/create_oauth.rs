use async_trait::async_trait;
use reqwest::StatusCode;
use serde::Serialize;
use thiserror::Error;

use crate::{
    akapi::{types::OAuthProvider, AkApiRoute, AkClient},
    resources::authentik_provider_oauth::crd,
};

pub struct CreateOAuthProvider;

#[async_trait]
impl AkApiRoute for CreateOAuthProvider {
    type Body = CreateOAuthProviderBody;
    type Response = OAuthProvider;
    type Error = CreateOAuthProviderError;

    #[instrument]
    async fn send(ak: &AkClient, body: Self::Body) -> Result<Self::Response, Self::Error> {
        let res = ak
            .post("/api/v3/providers/oauth2/")
            .json(&body)
            .send()
            .await?;

        match res.status() {
            StatusCode::CREATED => {
                let body: OAuthProvider = res.json().await?;

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
    #[error("Failed to send HTTP request: {0}")]
    ConnectionError(#[from] reqwest::Error),
}
