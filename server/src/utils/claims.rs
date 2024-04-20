use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Claims {
    iss: String,
    sub: String,
    exp: u64,
    iat: u64,
    jti: String,
    typ: String,
    azp: String,
    realm_access: RealmAccess,
    email_verified: bool,
    name: String,
    preferred_username: String,
    given_name: String,
    family_name: String,
    email: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct RealmAccess {
    roles: Vec<String>,
}
