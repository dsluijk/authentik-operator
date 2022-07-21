use async_trait::async_trait;
use reqwest::StatusCode;
use serde::Serialize;
use thiserror::Error;

use crate::akapi::{types::User, AkApiRoute, AkClient};

pub struct UpdateUser;

#[async_trait]
impl AkApiRoute for UpdateUser {
    type Body = UpdateUserBody;
    type Response = User;
    type Error = UpdateUserError;

    #[instrument]
    async fn send(ak: &AkClient, body: Self::Body) -> Result<Self::Response, Self::Error> {
        let res = ak
            .patch(&format!("/api/v3/core/users/{}/", body.id))
            .json(&body)
            .send()
            .await?;

        match res.status() {
            StatusCode::OK => {
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

#[derive(Debug, Serialize, Default)]
pub struct UpdateUserBody {
    pub id: usize,
    pub groups: Option<Vec<String>>,
}

#[derive(Error, Debug)]
pub enum UpdateUserError {
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error("Failed to send HTTP request: {0}")]
    ConnectionError(#[from] reqwest::Error),
}
