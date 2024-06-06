use serde::{Deserialize, Serialize};

use super::claims::Claims;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppData {
    pub user: UserData,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

impl UserData {
    pub fn from_claims(claims: &Claims) -> Self {
        Self {
            id: claims.sub(),
            email: claims.email(),
            name: claims.name(),
            preferred_username: claims.username(),
            given_name: claims.given_name(),
            family_name: claims.family_name(),
            roles: claims.roles(),
            is_admin: claims.is_admin(),
            is_super_admin: claims.is_super_admin(),
        }
    }
}
