use async_trait::async_trait;
use hyper::{Method, StatusCode};
use thiserror::Error;
use urlencoding::encode;

use crate::{
    akapi::{types::Flow, AkApiRoute, AkServer},
    error::AKApiError,
};

pub struct GetFlow;

#[async_trait]
impl AkApiRoute for GetFlow {
    type Body = String;
    type Response = Flow;
    type Error = GetFlowError;

    #[instrument]
    async fn send(
        api: &mut AkServer,
        api_key: &str,
        slug: Self::Body,
    ) -> Result<Self::Response, Self::Error> {
        let url = format!("/api/v3/flows/instances/{}/", encode(&slug));
        let res = api.send(Method::GET, url.as_str(), api_key, ()).await?;

        match res.status() {
            StatusCode::OK => {
                let bytes = hyper::body::to_bytes(res.into_body())
                    .await
                    .map_err(AKApiError::StreamError)?;
                let flow: Flow =
                    serde_json::from_slice(&bytes).map_err(AKApiError::SerializeError)?;

                Ok(flow)
            }
            code => Err(Self::Error::Unknown(format!(
                "Invalid status code {}",
                code
            ))),
        }
    }
}

#[derive(Error, Debug)]
pub enum GetFlowError {
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error(transparent)]
    RequestError(#[from] AKApiError),
}
