use async_trait::async_trait;
use hyper::{Method, StatusCode};
use serde::Deserialize;
use thiserror::Error;
use urlencoding::encode;

use crate::{
    akapi::{AkApiRoute, AkServer},
    error::AKApiError,
};

pub struct ViewToken;

#[async_trait]
impl AkApiRoute for ViewToken {
    type Body = String;
    type Response = String;
    type Error = ViewTokenError;

    async fn send(
        api: &mut AkServer,
        api_key: &str,
        ident: Self::Body,
    ) -> Result<Self::Response, Self::Error> {
        let res = api
            .send(
                Method::GET,
                format!("/api/v3/core/tokens/{}/view_key/", encode(&ident)).as_str(),
                api_key,
                (),
            )
            .await?;

        match res.status() {
            StatusCode::OK => {
                let bytes = hyper::body::to_bytes(res.into_body())
                    .await
                    .map_err(AKApiError::StreamError)?;
                let body: ViewTokenResponse =
                    serde_json::from_slice(&bytes).map_err(AKApiError::SerializeError)?;

                Ok(body.key)
            }
            StatusCode::NOT_FOUND => Err(Self::Error::NotFound),
            code => Err(Self::Error::Unknown(format!(
                "Invalid status code {}",
                code
            ))),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ViewTokenResponse {
    pub key: String,
}

#[derive(Error, Debug)]
pub enum ViewTokenError {
    #[error("The token was not found.")]
    NotFound,
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error("Failed send HTTP request: {0}")]
    RequestError(#[from] AKApiError),
}
