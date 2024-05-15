use anyhow::{anyhow, bail, Result};
use lru::LruCache;
use std::{fmt, num::NonZeroUsize, sync::Arc};
use tokio::sync::Mutex;

// use keycloak::{KeycloakAdmin, KeycloakAdminToken};

use crate::utils::{claims::Claims, config::Config};

use super::{jwk::Jwks, GrantType, JwksCache};

pub struct Keycloak {
    // admin: Arc<Mutex<KeycloakAdmin>>,
    jwk_cache: Arc<Mutex<LruCache<&'static str, JwksCache>>>,
    base_url: String,
    realm_id: String,
    client_id: String,
    client_secret: String,
}

impl Keycloak {
    pub async fn new() -> Result<Self> {
        let config = Config::default();

        Ok(Self {
            // admin: Arc::new(Mutex::new(admin)),
            jwk_cache: Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(1).unwrap()))),
            base_url: config.keycloak_url().to_owned(),
            realm_id: config.keycloak_realm().to_owned(),
            client_id: config.keycloak_client().to_owned(),
            client_secret: config.keycloak_client_secret().to_owned(),
        })
    }

    pub async fn exchange_token(
        &self,
        grant_type: GrantType<'_>,
        redirect_uri: &str,
    ) -> Result<super::TokenResponse> {
        let client = reqwest::Client::new();
        let base_url = &self.base_url;
        let realm_id = &self.realm_id;
        let client_secret = &self.client_secret;
        let token_url = format!("{base_url}/realms/{realm_id}/protocol/openid-connect/token");
        let form = [
            ("grant_type", grant_type.type_field_value()),
            ("client_id", &self.client_id),
            ("redirect_uri", redirect_uri),
            (grant_type.code_field_key(), grant_type.code_field_value()),
            ("client_secret", client_secret),
        ];

        let response = client.post(&token_url).form(&form).send().await;

        match response {
            Ok(response) => {
                if response.status().is_success() {
                    let token = response.json::<super::TokenResponse>().await?;
                    Ok(token)
                } else {
                    bail!(response.text().await?)
                }
            }
            Err(e) => bail!(e),
        }
    }

    pub async fn validate_token(&self, token: &str) -> Result<Claims> {
        let mut jwks = self.get_jwks().await?;

        let jwk = match jwks.match_kid(token) {
            Some(jwk) => jwk,
            None => {
                // If the token is not found in the JWKS, fetch the JWKS again and try to find the token
                jwks = self.fetch_jwks().await?;
                let jwk = jwks
                    .match_kid(token)
                    .ok_or(anyhow!("Token not found in JWKS"))?;

                // Update the cache with the new JWKS if a matching JWK is found
                self.jwk_cache
                    .lock()
                    .await
                    .put("c", JwksCache::from(jwks.clone()));
                jwk
            }
        };

        jwk.validate(token)
    }

    pub fn login_url(&self, redirect_uri: &str) -> String {
        let state = "asdf"; // generate_random_state();  // Implement this function to generate a random state
        let base_url = &self.base_url;
        let realm_id = &self.realm_id;
        let client_id = &self.client_id;

        let login_url = format!(
            "{base_url}/realms/{realm_id}/protocol/openid-connect/auth?client_id={client_id}&response_type=code&scope=openid&redirect_uri={redirect_uri}&state={state}",
        );
        login_url
    }

    pub async fn logout(&self, refresh_token: &str) -> Result<()> {
        let client = reqwest::Client::new();
        let base_url = &self.base_url;
        let realm_id = &self.realm_id;
        let client_secret = &self.client_secret;
        let token_url = format!("{base_url}/realms/{realm_id}/protocol/openid-connect/logout");
        let form = [
            ("client_id", &self.client_id),
            ("client_secret", client_secret),
            ("refresh_token", &refresh_token.to_owned()),
        ];

        client.post(&token_url).form(&form).send().await?;
        Ok(())
    }

    async fn get_jwks(&self) -> Result<Jwks> {
        let mut cache = self.jwk_cache.lock().await;

        match cache.get_mut("c") {
            Some(cache) => {
                let jwks = cache.get();
                let jwks = match jwks {
                    None => {
                        let result = self.fetch_jwks().await?;
                        cache.put(result.clone());
                        result
                    }
                    Some(jwks) => jwks.clone(),
                };

                Ok(jwks.clone())
            }
            None => {
                let result = self.fetch_jwks().await?;
                cache.put("c", JwksCache::from(result.clone()));
                Ok(result)
            }
        }
    }

    async fn fetch_jwks(&self) -> Result<Jwks> {
        let client = reqwest::Client::new();
        let base_url = &self.base_url;
        let realm_id = &self.realm_id;
        let jwks_url = format!("{base_url}/realms/{realm_id}/protocol/openid-connect/certs");

        let response = client.get(&jwks_url).send().await;

        match response {
            Ok(response) => {
                if response.status().is_success() {
                    let jwks = response.json::<Jwks>().await?;
                    Ok(jwks)
                } else {
                    dbg!(response.text().await?);
                    bail!("Failed to get JWKS")
                }
            }
            Err(e) => {
                dbg!(&e);
                bail!(e)
            }
        }
    }
}

impl fmt::Debug for Keycloak {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Keycloak")
            .field("internal_base_url", &self.base_url)
            .field("external_base_url", &self.base_url)
            .field("realm_id", &self.realm_id)
            .field("client_id", &self.client_id)
            // Optionally show whether admin is currently locked
            // .field("admin_locked", &self.admin.try_lock().is_err())
            .field("cache_locked", &self.jwk_cache.try_lock().is_err())
            .finish()
    }
}

// Implement Clone manually
impl Clone for Keycloak {
    fn clone(&self) -> Self {
        Keycloak {
            // admin: self.admin.clone(), // Clones the Arc, not the KeycloakAdmin
            jwk_cache: self.jwk_cache.clone(),
            base_url: self.base_url.clone(),
            realm_id: self.realm_id.clone(),
            client_id: self.client_id.clone(),
            client_secret: self.client_secret.clone(),
        }
    }
}
