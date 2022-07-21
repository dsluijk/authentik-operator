use async_trait::async_trait;
use reqwest::StatusCode;
use serde::Deserialize;
use thiserror::Error;

use crate::akapi::{types::ScopeMapping, AkApiRoute, AkClient};

pub struct FindScopeMapping;

#[async_trait]
impl AkApiRoute for FindScopeMapping {
    type Body = FindScopeMappingBody;
    type Response = Vec<ScopeMapping>;
    type Error = FindScopeMappingError;

    #[instrument]
    async fn send(ak: &AkClient, body: Self::Body) -> Result<Self::Response, Self::Error> {
        let mut query = vec![("page_size", "1000".to_string())];

        if let Some(name) = body.name {
            query.push(("name", name));
        }

        let res = ak
            .get("/api/v3/propertymappings/scope/")
            .query(&query)
            .send()
            .await?;

        match res.status() {
            StatusCode::OK => {
                let body: FindScopeMappingResponse = res.json().await?;

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
pub struct FindScopeMappingBody {
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FindScopeMappingResponse {
    pub results: Vec<ScopeMapping>,
}

#[derive(Error, Debug)]
pub enum FindScopeMappingError {
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error("Failed to send HTTP request: {0}")]
    ConnectionError(#[from] reqwest::Error),
}
