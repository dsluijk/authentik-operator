use async_trait::async_trait;
use hyper::{Method, StatusCode};
use serde::Deserialize;
use thiserror::Error;

use crate::{
    akapi::{types::User, AkApiRoute, AkServer},
    error::AKApiError,
};

pub struct Find;

#[async_trait]
impl AkApiRoute for Find {
    type Body = FindBody;
    type Response = Vec<User>;
    type Error = FindError;

    async fn send(
        mut api: AkServer,
        api_key: &str,
        body: Self::Body,
    ) -> Result<Self::Response, Self::Error> {
        let mut params = vec!["page_size=1000".to_string()];

        if let Some(name) = body.name {
            params.push(format!("name={}", name));
        }
        if let Some(username) = body.username {
            params.push(format!("username={}", username));
        }
        if let Some(uuid) = body.uuid {
            params.push(format!("uuid={}", uuid));
        }

        let url = format!("/api/v3/core/users/?{}", params.join("&"));
        let res = api.send(Method::GET, url.as_str(), api_key, ()).await?;

        match res.status() {
            StatusCode::OK => {
                let bytes = hyper::body::to_bytes(res.into_body())
                    .await
                    .map_err(AKApiError::StreamError)?;
                let body: FindResponse =
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
pub struct FindBody {
    pub name: Option<String>,
    pub username: Option<String>,
    pub uuid: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FindResponse {
    pub results: Vec<User>,
}

#[derive(Error, Debug)]
pub enum FindError {
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error("Failed send HTTP request: {0}")]
    RequestError(#[from] AKApiError),
}
