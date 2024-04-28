use serde::Serialize;

use super::claims::Claims;

#[derive(Debug, Clone, Default, Serialize)]
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
            id: claims.get_sub(),
            email: claims.get_email(),
            name: claims.get_name(),
            preferred_username: claims.get_username(),
            given_name: claims.get_given_name(),
            family_name: claims.get_family_name(),
            roles: claims.get_roles(),
            is_admin: claims.is_admin(),
            is_super_admin: claims.is_super_admin(),
        }
    }
}
