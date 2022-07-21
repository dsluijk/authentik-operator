use async_trait::async_trait;
use reqwest::StatusCode;
use thiserror::Error;

use crate::akapi::{AkApiRoute, AkClient};

pub struct DeleteFlow;

#[async_trait]
impl AkApiRoute for DeleteFlow {
    type Body = String;
    type Response = ();
    type Error = DeleteFlowError;

    #[instrument]
    async fn send(ak: &AkClient, slug: Self::Body) -> Result<Self::Response, Self::Error> {
        let res = ak
            .delete(&format!("/api/v3/flows/instances/{}/", slug))
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
pub enum DeleteFlowError {
    #[error("The given flow was not found.")]
    NotFound,
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error("Failed to send HTTP request: {0}")]
    ConnectionError(#[from] reqwest::Error),
}
