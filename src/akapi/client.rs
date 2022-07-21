use std::time::Duration;

use anyhow::Result;
use reqwest::{
    header::{HeaderMap, AUTHORIZATION},
    RequestBuilder,
};

#[derive(Debug)]
pub struct AkClient {
    client: reqwest::Client,
    host: String,
}

impl AkClient {
    pub fn new(api_key: &str, instance: &str, namespace: &str) -> Result<Self> {
        let user_agent = format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, format!("Bearer {}", api_key).parse()?);

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .user_agent(user_agent)
            .timeout(Duration::from_secs(120))
            .build()?;

        Ok(Self {
            client,
            host: format!("authentik-{}.{}", instance, namespace),
        })
    }

    pub fn get(&self, path: &str) -> RequestBuilder {
        self.client.get(format!("http://{}{}", self.host, path))
    }

    pub fn patch(&self, path: &str) -> RequestBuilder {
        self.client.patch(format!("http://{}{}", self.host, path))
    }

    pub fn post(&self, path: &str) -> RequestBuilder {
        self.client.post(format!("http://{}{}", self.host, path))
    }

    pub fn delete(&self, path: &str) -> RequestBuilder {
        self.client.delete(format!("http://{}{}", self.host, path))
    }
}
