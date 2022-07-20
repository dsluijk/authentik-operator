use serde::{Deserialize, Serialize};

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

#[derive(Debug, Deserialize)]
pub struct Provider {
    pub pk: usize,
    pub name: String,
    pub authorization_flow: String,
    pub property_mappings: Vec<String>,
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
