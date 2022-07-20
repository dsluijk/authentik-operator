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
    pub scopes: Vec<String>,
    #[validate(length(min = 1))]
    pub redirect_uris: Vec<String>,
    pub access_code_validity: Option<String>,
    pub token_validity: Option<String>,
    pub claims_in_token: Option<bool>,
    pub signing_key: Option<String>,
    pub subject_mode: Option<SubjectMode>,
    pub issuer_mode: Option<IssuerMode>,
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
