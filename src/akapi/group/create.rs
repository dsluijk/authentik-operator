use async_trait::async_trait;
use hyper::{Method, StatusCode};
use serde::Serialize;
use thiserror::Error;

use crate::{
    akapi::{types::Group, AkApiRoute, AkServer},
    error::AKApiError,
};

pub struct CreateGroup;

#[async_trait]
impl AkApiRoute for CreateGroup {
    type Body = CreateGroupBody;
    type Response = Group;
    type Error = CreateGroupError;

    async fn send(
        mut api: AkServer,
        api_key: &str,
        body: Self::Body,
    ) -> Result<Self::Response, Self::Error> {
        let res = api
            .send(Method::POST, "/api/v3/core/groups/", api_key, body)
            .await?;

        match res.status() {
            StatusCode::CREATED => {
                let bytes = hyper::body::to_bytes(res.into_body())
                    .await
                    .map_err(AKApiError::StreamError)?;
                let body: Group =
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
pub struct CreateGroupBody {
    pub name: String,
    pub is_superuser: bool,
    pub parent: String,
    pub users: Vec<usize>,
}

#[derive(Error, Debug)]
pub enum CreateGroupError {
    #[error("The group probably already exists!")]
    ExistsError,
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error("Failed send HTTP request: {0}")]
    RequestError(#[from] AKApiError),
}
