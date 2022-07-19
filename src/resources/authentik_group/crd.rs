use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[kube(
    kind = "AuthentikGroup",
    group = "ak.dany.dev",
    version = "v1",
    shortname = "akgroup",
    namespaced
)]
pub struct AuthentikGroupSpec {
    #[validate(length(min = 1))]
    pub authentik_instance: String,
    #[validate(length(min = 1))]
    pub name: String,
    pub superuser: Option<bool>,
    #[validate(length(min = 1))]
    pub parent: Option<String>,
}
