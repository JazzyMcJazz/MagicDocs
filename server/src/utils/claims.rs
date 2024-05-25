use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct JwtTokens {
    _id_token: String,
    access_token: String,
}

impl JwtTokens {
    pub fn new(_id_token: String, access_token: String) -> Self {
        Self {
            _id_token,
            access_token,
        }
    }
    pub fn _id_token(&self) -> &str {
        &self._id_token
    }
    pub fn access_token(&self) -> &str {
        &self.access_token
    }
}

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
    resource_access: ResourceAccess,
    email_verified: bool,
    name: String,
    preferred_username: String,
    given_name: String,
    family_name: String,
    email: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ResourceAccess {
    magicdocs: Option<Client>,
    #[serde(rename = "realm-management")]
    realm_management: Option<Client>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Client {
    roles: Vec<String>,
}

impl Claims {
    pub fn roles(&self) -> Vec<String> {
        match self.resource_access.magicdocs {
            Some(ref client) => client.roles.clone(),
            None => vec![],
        }
    }
    pub fn sub(&self) -> String {
        self.sub.clone()
    }
    pub fn name(&self) -> String {
        self.name.clone()
    }
    pub fn email(&self) -> String {
        self.email.clone()
    }
    pub fn username(&self) -> String {
        self.preferred_username.clone()
    }
    pub fn given_name(&self) -> String {
        self.given_name.clone()
    }
    pub fn family_name(&self) -> String {
        self.family_name.clone()
    }
    pub fn is_admin(&self) -> bool {
        self.is_super_admin()
            || match self.resource_access.magicdocs {
                Some(ref client) => client.roles.contains(&"admin".to_string()),
                None => false,
            }
    }
    pub fn is_super_admin(&self) -> bool {
        match self.resource_access.realm_management {
            Some(ref client) => !client.roles.is_empty(),
            None => false,
        }
    }
}
