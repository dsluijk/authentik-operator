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
