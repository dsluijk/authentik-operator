use async_trait::async_trait;
use reqwest::StatusCode;
use serde::Deserialize;
use thiserror::Error;

use crate::akapi::{types::User, AkApiRoute, AkClient};

pub struct GetSelf;

#[async_trait]
impl AkApiRoute for GetSelf {
    type Body = ();
    type Response = GetSelfResponse;
    type Error = GetSelfError;

    #[instrument]
    async fn send(ak: &AkClient, _body: Self::Body) -> Result<Self::Response, Self::Error> {
        let res = ak.get("/api/v3/core/users/me/").send().await?;

        match res.status() {
            StatusCode::OK => {
                let body: GetSelfResponse = res.json().await?;

                Ok(body)
            }
            StatusCode::FORBIDDEN => Err(Self::Error::Forbidden),
            code => Err(Self::Error::Unknown(format!(
                "Invalid status code {}",
                code
            ))),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GetSelfResponse {
    pub user: User,
    pub origional: Option<User>,
}

#[derive(Error, Debug)]
pub enum GetSelfError {
    #[error("Server denied our authentication.")]
    Forbidden,
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error("Failed to send HTTP request: {0}")]
    ConnectionError(#[from] reqwest::Error),
}
