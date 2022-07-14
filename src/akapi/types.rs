use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub pk: usize,
    pub uid: String,
    pub name: String,
    pub username: String,
    pub path: String,
    pub email: String,
    pub avatar: String,
    pub is_active: bool,
    pub is_superuser: bool,
    pub groups: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Group {
    pub pk: String,
    pub name: String,
    pub is_superuser: bool,
    pub parent: Option<String>,
    pub users: Vec<usize>,
}