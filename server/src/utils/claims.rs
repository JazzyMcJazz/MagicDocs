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
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Client {
    roles: Vec<String>,
}

impl Claims {
    pub fn get_roles(&self) -> Vec<String> {
        match self.resource_access.magicdocs {
            Some(ref client) => client.roles.clone(),
            None => vec![],
        }
    }
    pub fn get_sub(&self) -> String {
        self.sub.clone()
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    pub fn get_email(&self) -> String {
        self.email.clone()
    }
    pub fn get_username(&self) -> String {
        self.preferred_username.clone()
    }
    pub fn get_given_name(&self) -> String {
        self.given_name.clone()
    }
    pub fn get_family_name(&self) -> String {
        self.family_name.clone()
    }
    pub fn is_admin(&self) -> bool {
        match self.resource_access.magicdocs {
            Some(ref client) => {
                client.roles.contains(&"admin".to_string()) || self.is_super_admin()
            }
            None => false,
        }
    }
    pub fn is_super_admin(&self) -> bool {
        match self.resource_access.magicdocs {
            Some(ref client) => client.roles.contains(&"super_admin".to_string()),
            None => false,
        }
    }
}
