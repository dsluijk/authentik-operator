use async_trait::async_trait;

use crate::error::AKApiError;

pub mod group;
mod server;
pub mod token;
pub mod types;
pub mod user;

pub use server::AkServer;

pub static API_USER: &str = "ak-operator";
pub static TEMP_AUTH_TOKEN: &str = "AUTHENTIK_TEMP_AUTH_TOKEN";

pub fn service_group_name(instance: &str) -> String {
    format!("akOperator {} service group", instance)
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
