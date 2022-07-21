use anyhow::Result;
use async_trait::async_trait;

pub mod certificate;
pub mod flow;
pub mod group;
pub mod propertymappings;
pub mod provider;
pub mod stages;
pub mod token;
pub mod user;

pub mod auth;
mod client;
pub mod types;

pub use client::AkClient;

pub static API_USER: &str = "ak-operator";

pub fn service_group_name(instance: &str) -> String {
    format!("akOperator {} service group", instance)
}

pub fn token_identifier_name(instance: &str, purpose: &str) -> String {
    format!("ak-operator-{}__{}", instance, purpose)
}

#[async_trait]
pub trait AkApiRoute {
    type Body;
    type Response;
    type Error: From<reqwest::Error>;

    async fn send(ak: &AkClient, body: Self::Body) -> Result<Self::Response, Self::Error>;
}
