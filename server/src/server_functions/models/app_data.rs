use leptos::server_fn::serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AppData {
    pub user: UserData,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UserData {
    pub id: String,
    pub email: String,
    pub name: String,
    pub preferred_username: String,
    pub given_name: String,
    pub family_name: String,
    pub roles: Vec<String>,
    pub is_admin: bool,
    pub is_super_admin: bool,
}
