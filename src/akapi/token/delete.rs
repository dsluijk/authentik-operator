use async_trait::async_trait;
use reqwest::StatusCode;
use thiserror::Error;

use crate::akapi::{AkApiRoute, AkClient};

pub struct DeleteToken;

#[async_trait]
impl AkApiRoute for DeleteToken {
    type Body = String;
    type Response = ();
    type Error = DeleteTokenError;

    #[instrument]
    async fn send(ak: &AkClient, ident: Self::Body) -> Result<Self::Response, Self::Error> {
        let res = ak
            .delete(&format!("/api/v3/core/tokens/{}/", ident))
            .send()
            .await?;

        match res.status() {
            StatusCode::NO_CONTENT => Ok(()),
            StatusCode::NOT_FOUND => Err(Self::Error::NotFound),
            code => Err(Self::Error::Unknown(format!(
                "Invalid status code {}",
                code
            ))),
        }
    }
}

#[derive(Error, Debug)]
pub enum DeleteTokenError {
    #[error("The given token was not found.")]
    NotFound,
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error("Failed to send HTTP request: {0}")]
    ConnectionError(#[from] reqwest::Error),
}
