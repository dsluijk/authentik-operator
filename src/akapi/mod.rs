use async_trait::async_trait;

use crate::error::AKApiError;

mod server;
pub mod user;

pub use server::AkServer;

pub static API_USER: &str = "ak-operator";
pub static TEMP_AUTH_TOKEN: &str = "AUTHENTIK_TEMP_AUTH_TOKEN";

#[async_trait]
pub trait AkApiRoute {
    type Body;
    type Response;
    type Error: From<AKApiError>;

    async fn send(
        api: AkServer,
        api_key: &str,
        body: Self::Body,
    ) -> Result<Self::Response, Self::Error>;
}
