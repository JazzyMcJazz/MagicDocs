use lru::LruCache;
use std::{fmt, num::NonZeroUsize, sync::Arc};
use tokio::sync::Mutex;

// use keycloak::{KeycloakAdmin, KeycloakAdminToken};

use crate::utils::claims::Claims;

use super::{jwk::Jwks, GrantType, JwksCache};

pub struct Keycloak {
    // admin: Arc<Mutex<KeycloakAdmin>>,
    jwk_cache: Arc<Mutex<LruCache<&'static str, JwksCache>>>,
    internal_base_url: String,
    external_base_url: String,
    realm_id: String,
    client_id: String,
    client_secret: String,
}

impl Keycloak {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let internal_base_url =
            std::env::var("KEYCLOAK_INTERNAL_ADDR").expect("KEYCLOAK_INTERNAL_ADDR must be set");
        let external_base_url =
            std::env::var("KEYCLOAK_EXTERNAL_ADDR").expect("KEYCLOAK_EXTERNAL_ADDR must be set");
        // let user = std::env::var("KEYCLOAK_USER").expect("KEYCLOAK_USER must be set");
        // let password = std::env::var("KEYCLOAK_PASSWORD").expect("KEYCLOAK_PASSWORD must be set");
        let realm_id = std::env::var("KEYCLOAK_REALM").expect("KEYCLOAK_REALM must be set");
        let client_id = std::env::var("KEYCLOAK_CLIENT").expect("KEYCLOAK_CLIENT must be set");
        let client_secret =
            std::env::var("KEYCLOAK_CLIENT_SECRET").expect("KEYCLOAK_CLIENT_SECRET must be set");
        // let reqwest_client = reqwest::Client::new();
        // let admin_token =
        //     KeycloakAdminToken::acquire(&internal_base_url, &user, &password, &reqwest_client).await?;

        // let admin = KeycloakAdmin::new(&internal_base_url, admin_token, reqwest_client);

        Ok(Self {
            // admin: Arc::new(Mutex::new(admin)),
            jwk_cache: Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(1).unwrap()))),
            internal_base_url,
            external_base_url,
            realm_id,
            client_id,
            client_secret,
        })
    }

    pub async fn exchange_token(
        &self,
        grant_type: GrantType<'_>,
        redirect_uri: &str,
    ) -> Result<super::TokenResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let base_url = &self.internal_base_url;
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
                    dbg!(response.text().await?);
                    Err("Failed to exchange code for token".into())
                }
            }
            Err(e) => Err(e.into()),
        }
    }

    pub async fn validate_token(&self, token: &str) -> Result<Claims, Box<dyn std::error::Error>> {
        let mut jwks = self.get_jwks().await?;

        let jwk = match jwks.match_kid(token) {
            Some(jwk) => jwk,
            None => {
                // If the token is not found in the JWKS, fetch the JWKS again and try to find the token
                jwks = self.fetch_jwks().await?;
                let jwk = jwks.match_kid(token).ok_or("No matching JWK found")?;

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
        let base_url = &self.external_base_url;
        let realm_id = &self.realm_id;
        let client_id = &self.client_id;

        let login_url = format!(
            "{base_url}/realms/{realm_id}/protocol/openid-connect/auth?client_id={client_id}&response_type=code&scope=openid&redirect_uri={redirect_uri}&state={state}",
        );
        login_url
    }

    pub async fn logout(&self, refresh_token: &str) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let base_url = &self.internal_base_url;
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

    async fn get_jwks(&self) -> Result<Jwks, Box<dyn std::error::Error>> {
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

    async fn fetch_jwks(&self) -> Result<Jwks, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let base_url = &self.external_base_url;
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
                    Err("Failed to get JWKS".into())
                }
            }
            Err(e) => {
                dbg!(&e);
                Err(e.into())
            }
        }
    }
}

impl fmt::Debug for Keycloak {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Keycloak")
            .field("internal_base_url", &self.internal_base_url)
            .field("external_base_url", &self.external_base_url)
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
            internal_base_url: self.internal_base_url.clone(),
            external_base_url: self.external_base_url.clone(),
            realm_id: self.realm_id.clone(),
            client_id: self.client_id.clone(),
            client_secret: self.client_secret.clone(),
        }
    }
}
