use async_trait::async_trait;
use reqwest::StatusCode;
use serde::Deserialize;
use thiserror::Error;

use crate::akapi::{AkApiRoute, AkClient};

pub struct ViewToken;

#[async_trait]
impl AkApiRoute for ViewToken {
    type Body = String;
    type Response = String;
    type Error = ViewTokenError;

    #[instrument]
    async fn send(ak: &AkClient, ident: Self::Body) -> Result<Self::Response, Self::Error> {
        let res = ak
            .get(&format!("/api/v3/core/tokens/{}/view_key/", ident))
            .send()
            .await?;

        match res.status() {
            StatusCode::OK => {
                let body: ViewTokenResponse = res.json().await?;

                Ok(body.key)
            }
            StatusCode::NOT_FOUND => Err(Self::Error::NotFound),
            code => Err(Self::Error::Unknown(format!(
                "Invalid status code {}",
                code
            ))),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ViewTokenResponse {
    pub key: String,
}

#[derive(Error, Debug)]
pub enum ViewTokenError {
    #[error("The token was not found.")]
    NotFound,
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error("Failed to send HTTP request: {0}")]
    ConnectionError(#[from] reqwest::Error),
}
