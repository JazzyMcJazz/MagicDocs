#[derive(Debug)]
pub enum GrantType {
    AuthorizationCode(String),
    RefreshToken(String),
}

impl GrantType {
    pub fn type_field_value(&self) -> &str {
        match self {
            GrantType::AuthorizationCode(_) => "authorization_code",
            GrantType::RefreshToken(_) => "refresh_token",
        }
    }

    pub fn code_field_key(&self) -> &str {
        match self {
            GrantType::AuthorizationCode(_) => "code",
            GrantType::RefreshToken(_) => "refresh_token",
        }
    }

    pub fn code_field_value(&self) -> &str {
        match self {
            GrantType::AuthorizationCode(code) => code,
            GrantType::RefreshToken(code) => code,
        }
    }
}
