use async_trait::async_trait;
use reqwest::StatusCode;
use serde::Deserialize;
use thiserror::Error;

use crate::akapi::{types::Stage, AkApiRoute, AkClient};

pub struct FindStage;

#[async_trait]
impl AkApiRoute for FindStage {
    type Body = FindStageBody;
    type Response = Vec<Stage>;
    type Error = FindStageError;

    #[instrument]
    async fn send(ak: &AkClient, body: Self::Body) -> Result<Self::Response, Self::Error> {
        let mut query = vec![("page_size", "1000".to_string())];

        if let Some(name) = body.name {
            query.push(("name", name));
        }

        let res = ak.get("/api/v3/stages/all/").query(&query).send().await?;

        match res.status() {
            StatusCode::OK => {
                let body: FindStageResponse = res.json().await?;

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
pub struct FindStageBody {
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FindStageResponse {
    pub results: Vec<Stage>,
}

#[derive(Error, Debug)]
pub enum FindStageError {
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error("Failed to send HTTP request: {0}")]
    ConnectionError(#[from] reqwest::Error),
}
