use leptos::server_fn::serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppUser {
    pub created_timestamp: i128,
    pub email: String,
    pub email_verified: bool,
    pub enabled: bool,
    pub first_name: String,
    pub id: String,
    pub last_name: String,
    pub username: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppRole {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}
