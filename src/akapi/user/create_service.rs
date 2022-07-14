use async_trait::async_trait;
use hyper::{Method, StatusCode};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    akapi::{AkApiRoute, AkServer},
    error::AKApiError,
};

pub struct CreateServiceAccount;

#[async_trait]
impl AkApiRoute for CreateServiceAccount {
    type Body = CreateServiceAccountBody;
    type Response = CreateServiceAccountResponse;
    type Error = CreateServiceAccountError;

    async fn send(
        api: &mut AkServer,
        api_key: &str,
        body: Self::Body,
    ) -> Result<Self::Response, Self::Error> {
        let res = api
            .send(
                Method::POST,
                "/api/v3/core/users/service_account/",
                api_key,
                body,
            )
            .await?;

        match res.status() {
            StatusCode::OK => {
                let bytes = hyper::body::to_bytes(res.into_body())
                    .await
                    .map_err(AKApiError::StreamError)?;
                let body: CreateServiceAccountResponse =
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
pub struct CreateServiceAccountBody {
    pub name: String,
    pub create_group: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateServiceAccountResponse {
    pub username: String,
    pub user_uid: String,
    pub user_pk: usize,
    pub token: String,
}

#[derive(Error, Debug)]
pub enum CreateServiceAccountError {
    #[error("The user probably already exists!")]
    ExistsError,
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error("Failed send HTTP request: {0}")]
    RequestError(#[from] AKApiError),
}
