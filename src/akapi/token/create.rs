use async_trait::async_trait;
use reqwest::StatusCode;
use serde::Serialize;
use thiserror::Error;

use crate::akapi::{types::Token, AkApiRoute, AkClient};

pub struct CreateToken;

#[async_trait]
impl AkApiRoute for CreateToken {
    type Body = CreateTokenBody;
    type Response = Token;
    type Error = CreateTokenError;

    #[instrument]
    async fn send(ak: &AkClient, body: Self::Body) -> Result<Self::Response, Self::Error> {
        let res = ak.post("/api/v3/core/tokens/").json(&body).send().await?;

        match res.status() {
            StatusCode::CREATED => {
                let body: Token = res.json().await?;

                Ok(body)
            }
            StatusCode::BAD_REQUEST => Err(Self::Error::ExistsError),
            code => Err(Self::Error::Unknown(format!(
                "Invalid status code {}",
                code
            ))),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CreateTokenBody {
    pub identifier: String,
    pub intent: String,
    pub user: usize,
    pub description: String,
    pub expiring: bool,
}

#[derive(Error, Debug)]
pub enum CreateTokenError {
    #[error("The token already exists!")]
    ExistsError,
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error("Failed to send HTTP request: {0}")]
    ConnectionError(#[from] reqwest::Error),
}
