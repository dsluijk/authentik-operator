use async_trait::async_trait;
use hyper::{Method, StatusCode};
use serde::Deserialize;
use thiserror::Error;

use crate::{
    akapi::{types::User, AkApiRoute, AkServer},
    error::AKApiError,
};

pub struct GetSelf;

#[async_trait]
impl AkApiRoute for GetSelf {
    type Body = ();
    type Response = GetSelfResponse;
    type Error = GetSelfError;

    #[instrument]
    async fn send(
        api: &mut AkServer,
        api_key: &str,
        body: Self::Body,
    ) -> Result<Self::Response, Self::Error> {
        let res = api
            .send(Method::GET, "/api/v3/core/users/me/", api_key, body)
            .await?;

        match res.status() {
            StatusCode::OK => {
                let bytes = hyper::body::to_bytes(res.into_body())
                    .await
                    .map_err(AKApiError::StreamError)?;
                let body: GetSelfResponse =
                    serde_json::from_slice(&bytes).map_err(AKApiError::SerializeError)?;

                Ok(body)
            }
            StatusCode::FORBIDDEN => Err(Self::Error::Forbidden),
            code => Err(Self::Error::Unknown(format!(
                "Invalid status code {}",
                code
            ))),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GetSelfResponse {
    pub user: User,
    pub origional: Option<User>,
}

#[derive(Error, Debug)]
pub enum GetSelfError {
    #[error("Server denied our authentication.")]
    Forbidden,
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error(transparent)]
    RequestError(#[from] AKApiError),
}
