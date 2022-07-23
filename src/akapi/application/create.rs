use async_trait::async_trait;
use reqwest::StatusCode;
use thiserror::Error;

use crate::akapi::{types::Application, AkApiRoute, AkClient};

pub struct CreateApplication;

#[async_trait]
impl AkApiRoute for CreateApplication {
    type Body = Application;
    type Response = Application;
    type Error = CreateApplicationError;

    #[instrument]
    async fn send(ak: &AkClient, body: Self::Body) -> Result<Self::Response, Self::Error> {
        let res = ak
            .post("/api/v3/core/applications/")
            .json(&body)
            .send()
            .await?;

        match res.status() {
            StatusCode::CREATED => {
                let body: Application = res.json().await?;

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
pub enum CreateApplicationError {
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error("Failed to send HTTP request: {0}")]
    ConnectionError(#[from] reqwest::Error),
}
