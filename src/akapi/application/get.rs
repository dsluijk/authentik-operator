use async_trait::async_trait;
use reqwest::StatusCode;
use thiserror::Error;

use crate::akapi::{types::Application, AkApiRoute, AkClient};

pub struct GetApplication;

#[async_trait]
impl AkApiRoute for GetApplication {
    type Body = String;
    type Response = Option<Application>;
    type Error = GetApplicationError;

    #[instrument]
    async fn send(ak: &AkClient, slug: Self::Body) -> Result<Self::Response, Self::Error> {
        let res = ak
            .get(&format!("/api/v3/core/applications/{}/", slug))
            .send()
            .await?;

        match res.status() {
            StatusCode::OK => {
                let body: Application = res.json().await?;

                Ok(Some(body))
            }
            StatusCode::NOT_FOUND => Ok(None),
            code => Err(Self::Error::Unknown(format!(
                "Invalid status code {}",
                code
            ))),
        }
    }
}

#[derive(Error, Debug)]
pub enum GetApplicationError {
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error("Failed to send HTTP request: {0}")]
    ConnectionError(#[from] reqwest::Error),
}
