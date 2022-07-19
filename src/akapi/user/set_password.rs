use async_trait::async_trait;
use hyper::{Method, StatusCode};
use serde::Serialize;
use thiserror::Error;

use crate::{
    akapi::{AkApiRoute, AkServer},
    error::AKApiError,
};

pub struct SetPassword;

#[async_trait]
impl AkApiRoute for SetPassword {
    type Body = SetPasswordBody;
    type Response = ();
    type Error = SetPasswordError;

    #[instrument]
    async fn send(
        api: &mut AkServer,
        api_key: &str,
        body: Self::Body,
    ) -> Result<Self::Response, Self::Error> {
        let res = api
            .send(
                Method::POST,
                &format!("/api/v3/core/users/{}/set_password/", body.id),
                api_key,
                body,
            )
            .await?;

        match res.status() {
            StatusCode::NO_CONTENT => Ok(()),
            code => Err(Self::Error::Unknown(format!(
                "Invalid status code {}",
                code
            ))),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SetPasswordBody {
    pub id: usize,
    pub password: String,
}

#[derive(Error, Debug)]
pub enum SetPasswordError {
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error(transparent)]
    RequestError(#[from] AKApiError),
}
