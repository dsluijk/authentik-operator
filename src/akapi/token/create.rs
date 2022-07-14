use async_trait::async_trait;
use hyper::{Method, StatusCode};
use serde::Serialize;
use thiserror::Error;

use crate::{
    akapi::{types::Token, AkApiRoute, AkServer},
    error::AKApiError,
};

pub struct CreateToken;

#[async_trait]
impl AkApiRoute for CreateToken {
    type Body = CreateTokenBody;
    type Response = Token;
    type Error = CreateTokenError;

    async fn send(
        api: &mut AkServer,
        api_key: &str,
        body: Self::Body,
    ) -> Result<Self::Response, Self::Error> {
        let res = api
            .send(Method::POST, "/api/v3/core/tokens/", api_key, body)
            .await?;

        match res.status() {
            StatusCode::CREATED => {
                let bytes = hyper::body::to_bytes(res.into_body())
                    .await
                    .map_err(AKApiError::StreamError)?;
                let body: Token =
                    serde_json::from_slice(&bytes).map_err(AKApiError::SerializeError)?;

                Ok(body)
            }
            StatusCode::BAD_REQUEST => Err(Self::Error::ExistsError),
            code => Err(Self::Error::Unknown(format!(
                "Invalid status code {}",
                code
            ))),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CreateTokenBody {
    pub identifier: String,
    pub intent: String,
    pub user: usize,
    pub description: String,
    pub expiring: bool,
}

#[derive(Error, Debug)]
pub enum CreateTokenError {
    #[error("The token already exists!")]
    ExistsError,
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error("Failed send HTTP request: {0}")]
    RequestError(#[from] AKApiError),
}
