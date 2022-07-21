use async_trait::async_trait;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::akapi::{AkApiRoute, AkClient};

pub struct CreateServiceAccount;

#[async_trait]
impl AkApiRoute for CreateServiceAccount {
    type Body = CreateServiceAccountBody;
    type Response = CreateServiceAccountResponse;
    type Error = CreateServiceAccountError;

    #[instrument]
    async fn send(ak: &AkClient, body: Self::Body) -> Result<Self::Response, Self::Error> {
        let res = ak
            .post("/api/v3/core/users/service_account/")
            .json(&body)
            .send()
            .await?;

        match res.status() {
            StatusCode::OK => {
                let body: CreateServiceAccountResponse = res.json().await?;

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
pub struct CreateServiceAccountBody {
    pub name: String,
    pub create_group: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateServiceAccountResponse {
    pub username: String,
    pub user_uid: String,
    pub user_pk: usize,
    pub token: String,
}

#[derive(Error, Debug)]
pub enum CreateServiceAccountError {
    #[error("The user probably already exists!")]
    ExistsError,
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error("Failed to send HTTP request: {0}")]
    ConnectionError(#[from] reqwest::Error),
}
