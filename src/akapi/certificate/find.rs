use async_trait::async_trait;
use reqwest::StatusCode;
use serde::Deserialize;
use thiserror::Error;

use crate::akapi::{types::Certificate, AkApiRoute, AkClient};

pub struct FindCertificate;

#[async_trait]
impl AkApiRoute for FindCertificate {
    type Body = FindCertificateBody;
    type Response = Vec<Certificate>;
    type Error = FindCertificateError;

    #[instrument]
    async fn send(ak: &AkClient, body: Self::Body) -> Result<Self::Response, Self::Error> {
        let mut query = vec![("page_size", "1000".to_string())];

        if let Some(name) = body.name {
            query.push(("name", name));
        }

        if let Some(has_keys) = body.has_keys {
            query.push(("has_key", has_keys.to_string()));
        }

        let res = ak
            .get("/api/v3/crypto/certificatekeypairs/")
            .query(&query)
            .send()
            .await?;

        match res.status() {
            StatusCode::OK => {
                let body: FindCertificateResponse = res.json().await?;

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
pub struct FindCertificateBody {
    pub name: Option<String>,
    pub has_keys: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct FindCertificateResponse {
    pub results: Vec<Certificate>,
}

#[derive(Error, Debug)]
pub enum FindCertificateError {
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error("Failed to send HTTP request: {0}")]
    ConnectionError(#[from] reqwest::Error),
}
