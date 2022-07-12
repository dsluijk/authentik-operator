use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
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
    #[serde(default = "default_avatars")]
    pub avatars: String,
    #[serde(default = "default_image")]
    pub image: AuthentikImage,
    #[serde(default)]
    pub footer_links: Vec<AuthentikFooterLink>,
    pub ingress: Option<AuthentikIngress>,
    pub postgres: AuthentikPostgres,
    pub redis: AuthentikRedis,
    pub smtp: Option<AuthentikSmtp>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthentikImage {
    #[serde(default = "default_image_repo")]
    pub repository: String,
    #[serde(default = "default_image_tag")]
    pub tag: String,
    #[serde(default = "default_image_pullpolicy")]
    pub pull_policy: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthentikFooterLink {
    pub name: String,
    #[validate(url)]
    pub href: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthentikIngress {
    pub class_name: String,
    pub rules: Vec<AuthentikIngressRule>,
    pub tls: Option<Vec<AuthentikIngressTLS>>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthentikIngressRule {
    pub host: Option<String>,
    pub paths: Vec<AuthentikIngressPath>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthentikIngressPath {
    pub path: String,
    #[serde(default = "default_ingress_path_type")]
    pub path_type: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthentikIngressTLS {
    pub hosts: Option<Vec<String>>,
    pub secret_name: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthentikPostgres {
    pub host: String,
    #[serde(default = "default_postgres_port")]
    pub port: u16,
    pub database: String,
    pub username: String,
    #[serde(default = "default_postgres_password")]
    pub password: String,
    pub password_secret: Option<String>,
    pub password_secret_key: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthentikRedis {
    pub host: String,
    #[serde(default = "default_redis_port")]
    pub port: u16,
    pub password: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthentikSmtp {
    pub host: String,
    #[serde(default = "default_smtp_port")]
    pub port: u16,
    pub from: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default = "default_smtp_tls")]
    pub use_tls: bool,
    #[serde(default = "default_smtp_ssl")]
    pub use_ssl: bool,
    #[serde(default = "default_smtp_timeout")]
    pub timeout: u16,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthentikStatus {
    pub hidden: bool,
}

// -- Default value functions from here on.
fn default_avatars() -> String {
    "gravatar".to_string()
}

fn default_image() -> AuthentikImage {
    AuthentikImage {
        repository: default_image_repo(),
        tag: default_image_tag(),
        pull_policy: default_image_pullpolicy(),
    }
}

fn default_image_repo() -> String {
    "ghcr.io/goauthentik/server".to_string()
}

fn default_image_tag() -> String {
    "latest".to_string()
}

fn default_image_pullpolicy() -> String {
    "IfNotPresent".to_string()
}

fn default_ingress_path_type() -> String {
    "ImplementationSpecific".to_string()
}

fn default_postgres_port() -> u16 {
    5432
}

fn default_postgres_password() -> String {
    "postgres".to_string()
}

fn default_redis_port() -> u16 {
    6379
}

fn default_smtp_port() -> u16 {
    25
}

fn default_smtp_tls() -> bool {
    false
}

fn default_smtp_ssl() -> bool {
    false
}

fn default_smtp_timeout() -> u16 {
    10
}
