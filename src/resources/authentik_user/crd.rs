use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[kube(
    kind = "AuthentikUser",
    group = "ak.dany.dev",
    version = "v1",
    shortname = "akuser",
    namespaced
)]
pub struct AuthentikUserSpec {
    #[validate(length(min = 1))]
    pub authentik_instance: String,
    #[validate(length(min = 1))]
    pub username: String,
    pub display_name: String,
    #[validate(length(min = 1))]
    pub password: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(min = 1))]
    pub path: String,
    pub groups: Option<Vec<String>>,
}
