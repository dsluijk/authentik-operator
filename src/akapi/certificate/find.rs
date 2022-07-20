use async_trait::async_trait;
use hyper::{Method, StatusCode};
use serde::Deserialize;
use thiserror::Error;
use urlencoding::encode;

use crate::{
    akapi::{types::Certificate, AkApiRoute, AkServer},
    error::AKApiError,
};

pub struct FindCertificate;

#[async_trait]
impl AkApiRoute for FindCertificate {
    type Body = FindCertificateBody;
    type Response = Vec<Certificate>;
    type Error = FindCertificateError;

    #[instrument]
    async fn send(
        api: &mut AkServer,
        api_key: &str,
        body: Self::Body,
    ) -> Result<Self::Response, Self::Error> {
        let mut params = vec!["page_size=1000".to_string()];

        if let Some(name) = body.name {
            params.push(format!("name={}", encode(&name)));
        }

        if let Some(has_keys) = body.has_keys {
            params.push(format!("has_key={}", encode(&has_keys.to_string())));
        }

        let url = format!("/api/v3/crypto/certificatekeypairs/?{}", params.join("&"));
        let res = api.send(Method::GET, url.as_str(), api_key, ()).await?;

        match res.status() {
            StatusCode::OK => {
                let bytes = hyper::body::to_bytes(res.into_body())
                    .await
                    .map_err(AKApiError::StreamError)?;
                let body: FindCertificateResponse =
                    serde_json::from_slice(&bytes).map_err(AKApiError::SerializeError)?;

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
    #[error(transparent)]
    RequestError(#[from] AKApiError),
}
