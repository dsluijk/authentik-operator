use async_trait::async_trait;
use reqwest::StatusCode;
use serde::Serialize;
use thiserror::Error;

use crate::akapi::{types::Group, AkApiRoute, AkClient};

pub struct CreateGroup;

#[async_trait]
impl AkApiRoute for CreateGroup {
    type Body = CreateGroupBody;
    type Response = Group;
    type Error = CreateGroupError;

    #[instrument]
    async fn send(ak: &AkClient, body: Self::Body) -> Result<Self::Response, Self::Error> {
        let res = ak.post("/api/v3/core/groups/").json(&body).send().await?;

        match res.status() {
            StatusCode::CREATED => {
                let body: Group = res.json().await?;

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
pub struct CreateGroupBody {
    pub name: String,
    pub is_superuser: bool,
    pub parent: String,
    pub users: Vec<usize>,
}

#[derive(Error, Debug)]
pub enum CreateGroupError {
    #[error("The group probably already exists!")]
    ExistsError,
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error("Failed to send HTTP request: {0}")]
    ConnectionError(#[from] reqwest::Error),
}
