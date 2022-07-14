use async_trait::async_trait;
use hyper::{Method, StatusCode};
use thiserror::Error;

use crate::{
    akapi::{AkApiRoute, AkServer},
    error::AKApiError,
};

pub struct DeleteGroup;

#[async_trait]
impl AkApiRoute for DeleteGroup {
    type Body = String;
    type Response = ();
    type Error = DeleteGroupError;

    async fn send(
        mut api: AkServer,
        api_key: &str,
        uid: Self::Body,
    ) -> Result<Self::Response, Self::Error> {
        let res = api
            .send(
                Method::DELETE,
                format!("/api/v3/core/groups/{}/", uid).as_str(),
                api_key,
                (),
            )
            .await?;

        match res.status() {
            StatusCode::NO_CONTENT => Ok(()),
            StatusCode::BAD_REQUEST => Err(Self::Error::NotFound),
            code => Err(Self::Error::Unknown(format!(
                "Invalid status code {}",
                code
            ))),
        }
    }
}

#[derive(Error, Debug)]
pub enum DeleteGroupError {
    #[error("The given group was not found.")]
    NotFound,
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error("Failed send HTTP request: {0}")]
    RequestError(#[from] AKApiError),
}
