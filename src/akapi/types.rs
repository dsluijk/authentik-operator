use serde::{Deserialize, Serialize};

use crate::resources::authentik_application::crd::PolicyMode;

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub pk: usize,
    pub uid: String,
    pub name: String,
    pub username: String,
    pub path: Option<String>,
    pub email: String,
    pub avatar: String,
    pub is_active: bool,
    pub is_superuser: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Group {
    pub pk: String,
    pub name: String,
    pub is_superuser: bool,
    pub parent: Option<String>,
    pub users: Vec<usize>,
}

#[derive(Debug, Deserialize)]
pub struct Token {
    pub pk: String,
    pub manager: Option<String>,
    pub identifier: String,
    pub intent: String,
    pub user: usize,
    pub description: String,
    pub expiring: bool,
}

#[derive(Debug, Deserialize)]
pub struct Stage {
    pub pk: String,
    pub name: String,
    pub component: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Provider {
    pub pk: usize,
    pub name: String,
    pub authorization_flow: String,
    pub property_mappings: Option<Vec<String>>,
    pub component: String,
}

#[derive(Debug, Deserialize)]
pub struct OAuthProvider {
    pub pk: usize,
    pub name: String,
    pub authorization_flow: String,
    pub property_mappings: Option<Vec<String>>,
    pub client_id: String,
    pub client_secret: String,
    pub signing_key: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Flow {
    pub pk: String,
    pub name: String,
    pub slug: String,
    pub title: String,
    pub background: String,
}

#[derive(Debug, Deserialize)]
pub struct ScopeMapping {
    pub pk: String,
    pub name: String,
    pub managed: Option<String>,
    pub expression: String,
    pub component: String,
    pub scope_name: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct Certificate {
    pub pk: String,
    pub name: String,
    pub cert_expiry: String,
}

#[derive(Debug, Deserialize, Serialize, Default, PartialEq)]
pub struct Application {
    #[serde(skip_serializing)]
    pub pk: String,
    pub name: String,
    pub slug: String,
    pub provider: Option<usize>,
    #[serde(skip_serializing)]
    pub provider_obj: Option<Provider>,
    pub open_in_new_tab: Option<bool>,
    pub meta_launch_url: Option<String>,
    pub meta_description: Option<String>,
    pub meta_publisher: Option<String>,
    pub policy_engine_mode: Option<PolicyMode>,
    pub group: Option<String>,
}
