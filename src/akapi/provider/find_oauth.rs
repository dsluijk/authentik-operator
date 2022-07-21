use async_trait::async_trait;
use reqwest::StatusCode;
use serde::Deserialize;
use thiserror::Error;

use crate::akapi::{types::Provider, AkApiRoute, AkClient};

pub struct FindOAuthProvider;

#[async_trait]
impl AkApiRoute for FindOAuthProvider {
    type Body = FindOAuthProviderBody;
    type Response = Vec<Provider>;
    type Error = FindOAuthProviderError;

    #[instrument]
    async fn send(ak: &AkClient, body: Self::Body) -> Result<Self::Response, Self::Error> {
        let mut query = vec![("page_size", "1000".to_string())];

        if let Some(name) = body.name {
            query.push(("name", name));
        }

        let res = ak
            .get("/api/v3/providers/oauth2/")
            .query(&query)
            .send()
            .await?;

        match res.status() {
            StatusCode::OK => {
                let body: FindOAuthProviderResponse = res.json().await?;

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
pub struct FindOAuthProviderBody {
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FindOAuthProviderResponse {
    pub results: Vec<Provider>,
}

#[derive(Error, Debug)]
pub enum FindOAuthProviderError {
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error("Failed to send HTTP request: {0}")]
    ConnectionError(#[from] reqwest::Error),
}
