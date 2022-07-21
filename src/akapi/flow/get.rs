use async_trait::async_trait;
use reqwest::StatusCode;
use thiserror::Error;

use crate::akapi::{types::Flow, AkApiRoute, AkClient};

pub struct GetFlow;

#[async_trait]
impl AkApiRoute for GetFlow {
    type Body = String;
    type Response = Flow;
    type Error = GetFlowError;

    #[instrument]
    async fn send(ak: &AkClient, slug: Self::Body) -> Result<Self::Response, Self::Error> {
        let res = ak
            .get(&format!("/api/v3/flows/instances/{}/", slug))
            .send()
            .await?;

        match res.status() {
            StatusCode::OK => {
                let flow: Flow = res.json().await?;

                Ok(flow)
            }
            code => Err(Self::Error::Unknown(format!(
                "Invalid status code {}",
                code
            ))),
        }
    }
}

#[derive(Error, Debug)]
pub enum GetFlowError {
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error("Failed to send HTTP request: {0}")]
    ConnectionError(#[from] reqwest::Error),
}
