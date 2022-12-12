use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[kube(
    kind = "AuthentikOAuthProvider",
    group = "ak.dany.dev",
    version = "v1",
    shortname = "akoauth",
    namespaced
)]
pub struct AuthentikOAuthProviderSpec {
    #[validate(length(min = 1))]
    pub authentik_instance: String,
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(length(min = 1))]
    pub flow: String,
    pub client_type: ClientType,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub scopes: Vec<String>,
    #[validate(length(min = 1))]
    pub redirect_uris: Vec<String>,
    #[serde(default = "default_access_code_validity")]
    pub access_code_validity: String,
    #[serde(default = "default_token_validity")]
    pub token_validity: String,
    #[serde(default = "default_claims_in_token")]
    pub claims_in_token: bool,
    #[serde(default)]
    pub signing_key: Option<String>,
    #[serde(default = "default_subject_mode")]
    pub subject_mode: SubjectMode,
    #[serde(default = "default_issuer_mode")]
    pub issuer_mode: IssuerMode,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ClientType {
    Confidential,
    Public,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SubjectMode {
    HashedUserId,
    UserUsername,
    UserEmail,
    UserUpn,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IssuerMode {
    Global,
    PerProvider,
}

// -- Default value functions from here on.
fn default_access_code_validity() -> String {
    "minutes=1".to_string()
}

fn default_token_validity() -> String {
    "days=30".to_string()
}

fn default_claims_in_token() -> bool {
    true
}

fn default_subject_mode() -> SubjectMode {
    SubjectMode::HashedUserId
}

fn default_issuer_mode() -> IssuerMode {
    IssuerMode::PerProvider
}
