use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    access_token: String,
    refresh_token: String,
    id_token: String,
    token_type: String,
    expires_in: u32,
}
