use async_trait::async_trait;
use hyper::{Method, StatusCode};
use serde::Deserialize;
use thiserror::Error;
use urlencoding::encode;

use crate::{
    akapi::{types::Stage, AkApiRoute, AkServer},
    error::AKApiError,
};

pub struct FindStage;

#[async_trait]
impl AkApiRoute for FindStage {
    type Body = FindStageBody;
    type Response = Vec<Stage>;
    type Error = FindStageError;

    #[instrument]
    async fn send(
        api: &mut AkServer,
        api_key: &str,
        body: Self::Body,
    ) -> Result<Self::Response, Self::Error> {
        let mut params = vec!["page_size=1000".to_string()];

        if let Some(name) = body.name {
            params.push(format!("name={}", encode(&name)));
        }

        let url = format!("/api/v3/stages/all/?{}", params.join("&"));
        let res = api.send(Method::GET, url.as_str(), api_key, ()).await?;

        match res.status() {
            StatusCode::OK => {
                let bytes = hyper::body::to_bytes(res.into_body())
                    .await
                    .map_err(AKApiError::StreamError)?;
                let body: FindStageResponse =
                    serde_json::from_slice(&bytes).map_err(AKApiError::SerializeError)?;

                Ok(body.results)
            }
            code => Err(Self::Error::Unknown(format!(
                "Invalid status code {}",
                code
            ))),
        }
    }
}

#[derive(Debug, Default)]
pub struct FindStageBody {
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FindStageResponse {
    pub results: Vec<Stage>,
}

#[derive(Error, Debug)]
pub enum FindStageError {
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error(transparent)]
    RequestError(#[from] AKApiError),
}
