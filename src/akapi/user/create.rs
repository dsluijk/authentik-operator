use async_trait::async_trait;
use hyper::{Method, StatusCode};
use serde::Serialize;
use thiserror::Error;

use crate::{
    akapi::{types::User, AkApiRoute, AkServer},
    error::AKApiError,
};

pub struct CreateAccount;

#[async_trait]
impl AkApiRoute for CreateAccount {
    type Body = CreateAccountBody;
    type Response = User;
    type Error = CreateAccountError;

    #[instrument]
    async fn send(
        api: &mut AkServer,
        api_key: &str,
        body: Self::Body,
    ) -> Result<Self::Response, Self::Error> {
        let res = api
            .send(Method::POST, "/api/v3/core/users/", api_key, body)
            .await?;

        match res.status() {
            StatusCode::CREATED => {
                let bytes = hyper::body::to_bytes(res.into_body())
                    .await
                    .map_err(AKApiError::StreamError)?;
                let body: User =
                    serde_json::from_slice(&bytes).map_err(AKApiError::SerializeError)?;

                Ok(body)
            }
            code => Err(Self::Error::Unknown(format!(
                "Invalid status code {}",
                code
            ))),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CreateAccountBody {
    pub name: String,
    pub username: String,
    pub email: Option<String>,
    pub path: String,
    pub groups: Vec<String>,
}

#[derive(Error, Debug)]
pub enum CreateAccountError {
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error(transparent)]
    RequestError(#[from] AKApiError),
}
