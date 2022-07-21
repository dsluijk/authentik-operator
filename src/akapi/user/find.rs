use async_trait::async_trait;
use reqwest::StatusCode;
use serde::Deserialize;
use thiserror::Error;

use crate::akapi::{types::User, AkApiRoute, AkClient};

pub struct Find;

#[async_trait]
impl AkApiRoute for Find {
    type Body = FindBody;
    type Response = Vec<User>;
    type Error = FindError;

    #[instrument]
    async fn send(ak: &AkClient, body: Self::Body) -> Result<Self::Response, Self::Error> {
        let mut query = vec![("page_size", "1000".to_string())];

        if let Some(name) = body.name {
            query.push(("name", name));
        }
        if let Some(username) = body.username {
            query.push(("username", username));
        }
        if let Some(uuid) = body.uuid {
            query.push(("uuid", uuid));
        }

        let res = ak.get("/api/v3/core/users/").query(&query).send().await?;

        match res.status() {
            StatusCode::OK => {
                let body: FindResponse = res.json().await?;

                Ok(body.results)
            }
            code => Err(Self::Error::Unknown(format!(
                "Invalid status code {}",
                code
            ))),
        }
    }
}

#[derive(Debug, Default)]
pub struct FindBody {
    pub name: Option<String>,
    pub username: Option<String>,
    pub uuid: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FindResponse {
    pub results: Vec<User>,
}

#[derive(Error, Debug)]
pub enum FindError {
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error("Failed to send HTTP request: {0}")]
    ConnectionError(#[from] reqwest::Error),
}
