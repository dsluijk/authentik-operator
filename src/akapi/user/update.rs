use async_trait::async_trait;
use hyper::{Method, StatusCode};
use serde::Serialize;
use thiserror::Error;

use crate::{
    akapi::{types::User, AkApiRoute, AkServer},
    error::AKApiError,
};

pub struct UpdateUser;

#[async_trait]
impl AkApiRoute for UpdateUser {
    type Body = UpdateUserBody;
    type Response = User;
    type Error = UpdateUserError;

    #[instrument]
    async fn send(
        api: &mut AkServer,
        api_key: &str,
        body: Self::Body,
    ) -> Result<Self::Response, Self::Error> {
        let res = api
            .send(
                Method::PATCH,
                &format!("/api/v3/core/users/{}/", body.id),
                api_key,
                body,
            )
            .await?;

        match res.status() {
            StatusCode::OK => {
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

#[derive(Debug, Serialize, Default)]
pub struct UpdateUserBody {
    pub id: usize,
    pub groups: Option<Vec<String>>,
}

#[derive(Error, Debug)]
pub enum UpdateUserError {
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error(transparent)]
    RequestError(#[from] AKApiError),
}
