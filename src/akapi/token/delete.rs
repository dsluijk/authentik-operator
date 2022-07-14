use async_trait::async_trait;
use hyper::{Method, StatusCode};
use thiserror::Error;
use urlencoding::encode;

use crate::{
    akapi::{AkApiRoute, AkServer},
    error::AKApiError,
};

pub struct DeleteToken;

#[async_trait]
impl AkApiRoute for DeleteToken {
    type Body = String;
    type Response = ();
    type Error = DeleteTokenError;

    async fn send(
        api: &mut AkServer,
        api_key: &str,
        ident: Self::Body,
    ) -> Result<Self::Response, Self::Error> {
        let res = api
            .send(
                Method::DELETE,
                format!("/api/v3/core/tokens/{}/", encode(&ident)).as_str(),
                api_key,
                (),
            )
            .await?;

        match res.status() {
            StatusCode::NO_CONTENT => Ok(()),
            StatusCode::NOT_FOUND => Err(Self::Error::NotFound),
            code => Err(Self::Error::Unknown(format!(
                "Invalid status code {}",
                code
            ))),
        }
    }
}

#[derive(Error, Debug)]
pub enum DeleteTokenError {
    #[error("The given token was not found.")]
    NotFound,
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error("Failed send HTTP request: {0}")]
    RequestError(#[from] AKApiError),
}
