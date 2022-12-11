use async_trait::async_trait;
use reqwest::StatusCode;
use thiserror::Error;

use crate::akapi::{types::OAuthProvider, AkApiRoute, AkClient};

pub struct PatchOAuthProvider;

#[async_trait]
impl AkApiRoute for PatchOAuthProvider {
    type Body = OAuthProvider;
    type Response = OAuthProvider;
    type Error = PatchOAuthProviderError;

    #[instrument]
    async fn send(ak: &AkClient, body: Self::Body) -> Result<Self::Response, Self::Error> {
        let res = ak
            .patch(&format!("/api/v3/providers/oauth2/{}/", body.pk))
            .json(&body)
            .send()
            .await?;

        match res.status() {
            StatusCode::OK => {
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

#[derive(Error, Debug)]
pub enum PatchOAuthProviderError {
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error("Failed to send HTTP request: {0}")]
    ConnectionError(#[from] reqwest::Error),
}
