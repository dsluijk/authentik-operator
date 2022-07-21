use async_trait::async_trait;
use reqwest::StatusCode;
use thiserror::Error;

use crate::akapi::{AkApiRoute, AkClient};

pub struct DeleteGroup;

#[async_trait]
impl AkApiRoute for DeleteGroup {
    type Body = String;
    type Response = ();
    type Error = DeleteGroupError;

    #[instrument]
    async fn send(ak: &AkClient, uid: Self::Body) -> Result<Self::Response, Self::Error> {
        let res = ak
            .delete(&format!("/api/v3/core/groups/{}/", uid))
            .send()
            .await?;

        match res.status() {
            StatusCode::NO_CONTENT => Ok(()),
            StatusCode::BAD_REQUEST => Err(Self::Error::NotFound),
            code => Err(Self::Error::Unknown(format!(
                "Invalid status code {}",
                code
            ))),
        }
    }
}

#[derive(Error, Debug)]
pub enum DeleteGroupError {
    #[error("The given group was not found.")]
    NotFound,
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error("Failed to send HTTP request: {0}")]
    ConnectionError(#[from] reqwest::Error),
}
