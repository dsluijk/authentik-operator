use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(
    kind = "Authentik",
    group = "ak.dany.dev",
    version = "v1",
    status = "AuthentikStatus",
    plural = "authentik",
    shortname = "ak",
    namespaced
)]
pub struct AuthentikSpec {
    pub secret_key: Option<String>,
    pub log_level: Option<String>,
    pub avatars: Option<String>,
    pub footer_links: Option<Vec<AuthentikFooterLink>>,
    pub postgres: AuthentikPostgresSpec,
    pub redis: AuthentikRedisSpec,
    pub smtp: Option<AuthentikSmtpSpec>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct AuthentikFooterLink {
    pub name: String,
    #[validate(url)]
    pub href: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct AuthentikPostgresSpec {
    pub host: String,
    pub port: Option<u16>,
    pub name: String,
    pub username: String,
    pub password: Option<String>,
    pub password_secret: Option<String>,
    pub password_secret_key: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct AuthentikRedisSpec {
    pub host: String,
    pub port: Option<u16>,
    pub password: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct AuthentikSmtpSpec {
    pub host: String,
    pub port: Option<u16>,
    pub from: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub use_tls: Option<bool>,
    pub use_ssl: Option<bool>,
    pub timeout: Option<u16>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct AuthentikStatus {
    pub hidden: bool,
}
