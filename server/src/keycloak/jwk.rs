use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use std::str::FromStr;

use crate::utils::claims::Claims;

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Jwk {
    kid: String,
    kty: String,
    alg: String,
    #[serde(rename = "use")]
    key_use: String,
    n: String,
    e: String,
    x5c: Vec<String>,
    x5t: String,
    #[serde(rename = "x5t#S256")]
    x5t_s256: String,
}

impl Jwk {
    pub fn validate(&self, token: &str) -> Result<Claims, Box<dyn std::error::Error>> {
        let decoding_key = DecodingKey::from_rsa_components(&self.n, &self.e)?;
        let alg = Algorithm::from_str(self.alg.as_str())?;
        let mut validation = Validation::new(alg);
        validation.set_audience(&["account"]);

        let result = jsonwebtoken::decode::<Claims>(token, &decoding_key, &validation)?;
        Ok(result.claims)
    }
}
