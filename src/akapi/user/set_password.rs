use async_trait::async_trait;
use reqwest::StatusCode;
use serde::Serialize;
use thiserror::Error;

use crate::akapi::{AkApiRoute, AkClient};

pub struct SetPassword;

#[async_trait]
impl AkApiRoute for SetPassword {
    type Body = SetPasswordBody;
    type Response = ();
    type Error = SetPasswordError;

    #[instrument]
    async fn send(ak: &AkClient, body: Self::Body) -> Result<Self::Response, Self::Error> {
        let res = ak
            .post(&format!("/api/v3/core/users/{}/set_password/", body.id))
            .json(&body)
            .send()
            .await?;

        match res.status() {
            StatusCode::NO_CONTENT => Ok(()),
            code => Err(Self::Error::Unknown(format!(
                "Invalid status code {}",
                code
            ))),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SetPasswordBody {
    pub id: usize,
    pub password: String,
}

#[derive(Error, Debug)]
pub enum SetPasswordError {
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error("Failed to send HTTP request: {0}")]
    ConnectionError(#[from] reqwest::Error),
}
