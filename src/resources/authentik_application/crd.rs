use kube::CustomResource;
use lazy_static::lazy_static;
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

lazy_static! {
    static ref SLUG_VALIDATOR: Regex = Regex::new(r"^[-a-zA-Z0-9_]+$").unwrap();
}

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[kube(
    kind = "AuthentikApplication",
    group = "ak.dany.dev",
    version = "v1",
    shortname = "akapp",
    namespaced
)]
pub struct AuthentikApplicationSpec {
    #[validate(length(min = 1))]
    pub authentik_instance: String,
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(regex = "SLUG_VALIDATOR")]
    pub slug: String,
    #[validate(length(min = 1))]
    pub provider: String,
    #[validate(length(min = 1))]
    pub group: Option<String>,
    #[serde(default = "default_policy")]
    pub policy_mode: PolicyMode,
    #[serde(default)]
    pub ui: AuthentikApplicationUI,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PolicyMode {
    All,
    Any,
}

#[derive(Deserialize, Default, Serialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthentikApplicationUI {
    #[serde(default)]
    pub new_tab: bool,
    #[validate(url)]
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default = "default_ui_icon")]
    pub icon: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub publisher: String,
}

fn default_policy() -> PolicyMode {
    PolicyMode::Any
}

fn default_ui_icon() -> String {
    "fa://fa-eye".to_string()
}
