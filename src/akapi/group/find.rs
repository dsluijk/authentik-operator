use async_trait::async_trait;
use hyper::{Method, StatusCode};
use serde::Deserialize;
use thiserror::Error;
use url::form_urlencoded;

use crate::{
    akapi::{types::Group, AkApiRoute, AkServer},
    error::AKApiError,
};

pub struct FindGroup;

#[async_trait]
impl AkApiRoute for FindGroup {
    type Body = FindGroupBody;
    type Response = Vec<Group>;
    type Error = FindGroupError;

    async fn send(
        mut api: AkServer,
        api_key: &str,
        body: Self::Body,
    ) -> Result<Self::Response, Self::Error> {
        let url = format!("/api/v3/core/groups/?{}", Self::make_params(body));
        let res = api.send(Method::GET, url.as_str(), api_key, ()).await?;

        match res.status() {
            StatusCode::OK => {
                let bytes = hyper::body::to_bytes(res.into_body())
                    .await
                    .map_err(AKApiError::StreamError)?;
                let body: FindGroupResponse =
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

impl FindGroup {
    fn make_params(body: FindGroupBody) -> String {
        let mut params = form_urlencoded::Serializer::new(String::new());
        params.append_pair("page_size", "1000");

        if let Some(name) = body.name {
            params.append_pair("name", &name.clone());
        }

        params.finish().to_string()
    }
}

#[derive(Debug, Default)]
pub struct FindGroupBody {
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FindGroupResponse {
    pub results: Vec<Group>,
}

#[derive(Error, Debug)]
pub enum FindGroupError {
    #[error("An unknown error occured ({0}).")]
    Unknown(String),
    #[error("Failed send HTTP request: {0}")]
    RequestError(#[from] AKApiError),
}
