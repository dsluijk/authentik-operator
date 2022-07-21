use async_trait::async_trait;
use reqwest::StatusCode;
use serde::Serialize;
use thiserror::Error;

use crate::akapi::{types::User, AkApiRoute, AkClient};

pub struct CreateAccount;

#[async_trait]
impl AkApiRoute for CreateAccount {
    type Body = CreateAccountBody;
    type Response = User;
    type Error = CreateAccountError;

    #[instrument]
    async fn send(ak: &AkClient, body: Self::Body) -> Result<Self::Response, Self::Error> {
        let res = ak.post("/api/v3/core/users/").json(&body).send().await?;

        match res.status() {
            StatusCode::CREATED => {
                let body: User = res.json().await?;

                Ok(body)
            }
            code => Err(Self::Error::Unknown(format!(
                "Invalid status code {}",
                code
            ))),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CreateAccountBody {
    pub name: String,
    pub username: String,
    pub email: Option<String>,
    pub path: String,
    pub groups: Vec<String>,
}

#[derive(Error, Debug)]
pub enum CreateAccountError {
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error("Failed to send HTTP request: {0}")]
    ConnectionError(#[from] reqwest::Error),
}
