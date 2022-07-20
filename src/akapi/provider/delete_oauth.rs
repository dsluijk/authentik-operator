use async_trait::async_trait;
use hyper::{Method, StatusCode};
use thiserror::Error;

use crate::{
    akapi::{AkApiRoute, AkServer},
    error::AKApiError,
};

pub struct DeleteOAuthProvider;

#[async_trait]
impl AkApiRoute for DeleteOAuthProvider {
    type Body = usize;
    type Response = ();
    type Error = DeleteOAuthProviderError;

    #[instrument]
    async fn send(
        api: &mut AkServer,
        api_key: &str,
        slug: Self::Body,
    ) -> Result<Self::Response, Self::Error> {
        let res = api
            .send(
                Method::DELETE,
                format!("/api/v3/providers/oauth2/{}/", slug).as_str(),
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
pub enum DeleteOAuthProviderError {
    #[error("The given oauth provider was not found.")]
    NotFound,
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error(transparent)]
    RequestError(#[from] AKApiError),
}
