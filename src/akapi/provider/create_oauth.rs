use async_trait::async_trait;
use reqwest::StatusCode;
use thiserror::Error;

use crate::akapi::{types::OAuthProvider, AkApiRoute, AkClient};

pub struct CreateOAuthProvider;

#[async_trait]
impl AkApiRoute for CreateOAuthProvider {
    type Body = OAuthProvider;
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

#[derive(Error, Debug)]
pub enum CreateOAuthProviderError {
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error("Failed to send HTTP request: {0}")]
    ConnectionError(#[from] reqwest::Error),
}
