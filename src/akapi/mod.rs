use async_trait::async_trait;

use crate::error::AKApiError;

pub mod certificate;
pub mod flow;
pub mod group;
pub mod propertymappings;
pub mod provider;
pub mod stages;
pub mod token;
pub mod user;

pub mod auth;
mod server;
pub mod types;

pub use server::AkServer;

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
    type Error: From<AKApiError>;

    async fn send(
        api: &mut AkServer,
        api_key: &str,
        body: Self::Body,
    ) -> Result<Self::Response, Self::Error>;
}
