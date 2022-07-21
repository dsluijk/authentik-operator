use async_trait::async_trait;
use reqwest::StatusCode;
use serde::Deserialize;
use thiserror::Error;

use crate::akapi::{types::Group, AkApiRoute, AkClient};

pub struct FindGroup;

#[async_trait]
impl AkApiRoute for FindGroup {
    type Body = FindGroupBody;
    type Response = Vec<Group>;
    type Error = FindGroupError;

    #[instrument]
    async fn send(ak: &AkClient, body: Self::Body) -> Result<Self::Response, Self::Error> {
        let mut query = vec![("page_size", "1000".to_string())];

        if let Some(name) = body.name {
            query.push(("name", name));
        }

        let res = ak.get("/api/v3/core/groups/").query(&query).send().await?;

        match res.status() {
            StatusCode::OK => {
                let body: FindGroupResponse = res.json().await?;

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
pub struct FindGroupBody {
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FindGroupResponse {
    pub results: Vec<Group>,
}

#[derive(Error, Debug)]
pub enum FindGroupError {
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error("Failed to send HTTP request: {0}")]
    ConnectionError(#[from] reqwest::Error),
}
