use async_trait::async_trait;
use reqwest::StatusCode;
use serde::Deserialize;
use thiserror::Error;

use crate::akapi::{types::Provider, AkApiRoute, AkClient};

pub struct FindProvider;

#[async_trait]
impl AkApiRoute for FindProvider {
    type Body = FindProviderBody;
    type Response = Vec<Provider>;
    type Error = FindProviderError;

    #[instrument]
    async fn send(ak: &AkClient, body: Self::Body) -> Result<Self::Response, Self::Error> {
        let mut query = vec![("page_size", "1000".to_string())];

        if let Some(search) = body.search {
            query.push(("search", search));
        }

        let res = ak
            .get("/api/v3/providers/all/")
            .query(&query)
            .send()
            .await?;

        match res.status() {
            StatusCode::OK => {
                let body: FindProviderResponse = res.json().await?;

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
pub struct FindProviderBody {
    pub search: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FindProviderResponse {
    pub results: Vec<Provider>,
}

#[derive(Error, Debug)]
pub enum FindProviderError {
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error("Failed to send HTTP request: {0}")]
    ConnectionError(#[from] reqwest::Error),
}
