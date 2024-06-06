use anyhow::{anyhow, bail, Result};
use lru::LruCache;
use std::{fmt, num::NonZeroUsize, sync::Arc};
use tokio::sync::Mutex;

use crate::{
    models::CreateRole,
    utils::claims::{Claims, JwtTokens},
    CONFIG,
};

use super::{
    jwk::Jwks,
    response_types::{KeycloakRole, KeycloakRoleMapping, KeycloakUser},
    GrantType, JwksCache,
};

pub struct Keycloak {
    jwk_cache: Arc<Mutex<LruCache<&'static str, JwksCache>>>,
}

impl Default for Keycloak {
    fn default() -> Self {
        Self {
            jwk_cache: Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(1).unwrap()))),
        }
    }
}

impl Keycloak {
    pub async fn exchange_token(
        &self,
        grant_type: GrantType<'_>,
        redirect_uri: &str,
    ) -> Result<super::TokenResponse> {
        let client = reqwest::Client::new();
        let base_url = CONFIG.keycloak_url();
        let realm_id = CONFIG.keycloak_realm();
        let token_url = format!("{base_url}/realms/{realm_id}/protocol/openid-connect/token");
        let form = [
            ("grant_type", grant_type.type_field_value()),
            ("client_id", CONFIG.keycloak_client_name()),
            ("redirect_uri", redirect_uri),
            (grant_type.code_field_key(), grant_type.code_field_value()),
            ("client_secret", CONFIG.keycloak_client_secret()),
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
        let base_url = CONFIG.keycloak_url();
        let realm_id = CONFIG.keycloak_realm();
        let client_id = CONFIG.keycloak_client_name();

        let login_url = format!(
            "{base_url}/realms/{realm_id}/protocol/openid-connect/auth?client_id={client_id}&response_type=code&scope=openid&redirect_uri={redirect_uri}&state={state}",
        );
        login_url
    }

    pub async fn logout(&self, refresh_token: &str) -> Result<()> {
        let client = reqwest::Client::new();
        let base_url = CONFIG.keycloak_url();
        let realm_id = CONFIG.keycloak_realm();
        let client_secret = CONFIG.keycloak_client_secret();
        let token_url = format!("{base_url}/realms/{realm_id}/protocol/openid-connect/logout");
        let form = [
            ("client_id", CONFIG.keycloak_client_name()),
            ("client_secret", client_secret),
            ("refresh_token", refresh_token),
        ];

        client.post(&token_url).form(&form).send().await?;
        Ok(())
    }

    pub async fn get_users(tokens: &JwtTokens, search: Option<&str>) -> Result<Vec<KeycloakUser>> {
        let client = reqwest::Client::new();
        let base_url = CONFIG.keycloak_url();
        let realm_id = CONFIG.keycloak_realm();
        let url = format!("{base_url}/admin/realms/{realm_id}/users");
        let mut url = url;
        if let Some(search) = search {
            url.push_str(&format!("?search={}", search));
        }

        let res = client
            .get(&url)
            .bearer_auth(tokens.access_token())
            .send()
            .await?;

        let users = res.json::<Vec<KeycloakUser>>().await?;
        Ok(users)
    }

    pub async fn get_user(tokens: &JwtTokens, id: &str) -> Result<KeycloakUser> {
        let client = reqwest::Client::new();
        let base_url = CONFIG.keycloak_url();
        let realm_id = CONFIG.keycloak_realm();
        let url = format!("{base_url}/admin/realms/{realm_id}/users/{id}");
        let res = client
            .get(&url)
            .bearer_auth(tokens.access_token())
            .send()
            .await?;

        let user = res.json::<KeycloakUser>().await?;
        Ok(user)
    }

    pub async fn get_user_roles(tokens: &JwtTokens, id: &str) -> Result<Vec<KeycloakRole>> {
        let client = reqwest::Client::new();
        let base_url = CONFIG.keycloak_url();
        let realm_id = CONFIG.keycloak_realm();
        let url = format!("{base_url}/admin/realms/{realm_id}/users/{id}/role-mappings");
        let res = client
            .get(&url)
            .bearer_auth(tokens.access_token())
            .send()
            .await?;

        let data = match res.json::<KeycloakRoleMapping>().await {
            Ok(data) => data,
            Err(e) => {
                dbg!(e);
                bail!("Failed to get user roles")
            }
        };
        let roles = data
            .client_roles(CONFIG.keycloak_client_name())
            .unwrap_or_default();
        Ok(roles)
    }

    pub async fn get_user_available_roles(
        tokens: &JwtTokens,
        user_id: &str,
    ) -> Result<Vec<KeycloakRole>> {
        let client = reqwest::Client::new();
        let base_url = CONFIG.keycloak_url();
        let realm_id = CONFIG.keycloak_realm();
        let client_uuid = CONFIG.keycloak_client_uuid();
        let url = format!("{base_url}/admin/realms/{realm_id}/users/{user_id}/role-mappings/clients/{client_uuid}/available");
        let res = client
            .get(&url)
            .bearer_auth(tokens.access_token())
            .send()
            .await?;

        let roles = match res.json::<Vec<KeycloakRole>>().await {
            Ok(data) => data,
            Err(e) => {
                tracing::error!("Error getting user roles: {:?}", e);
                bail!("Failed to get user roles")
            }
        };

        Ok(roles)
    }

    pub async fn grant_user_roles(
        tokens: &JwtTokens,
        user_id: &str,
        roles: &Vec<KeycloakRole>,
    ) -> Result<()> {
        let client = reqwest::Client::new();
        let base_url = CONFIG.keycloak_url();
        let realm_id = CONFIG.keycloak_realm();
        let client_uuid = CONFIG.keycloak_client_uuid();
        let url = format!("{base_url}/admin/realms/{realm_id}/users/{user_id}/role-mappings/clients/{client_uuid}");

        let res = client
            .post(&url)
            .bearer_auth(tokens.access_token())
            .json(&roles)
            .send()
            .await?;

        if !res.status().is_success() {
            bail!("Failed to grant user roles: {}", res.text().await?)
        }

        Ok(())
    }

    pub async fn revoke_user_roles(
        tokens: &JwtTokens,
        user_id: &str,
        roles: &Vec<KeycloakRole>,
    ) -> Result<()> {
        let client = reqwest::Client::new();
        let base_url = CONFIG.keycloak_url();
        let realm_id = CONFIG.keycloak_realm();
        let client_uuid = CONFIG.keycloak_client_uuid();
        let url = format!("{base_url}/admin/realms/{realm_id}/users/{user_id}/role-mappings/clients/{client_uuid}");

        let res = client
            .delete(&url)
            .bearer_auth(tokens.access_token())
            .json(&roles)
            .send()
            .await?;

        if !res.status().is_success() {
            bail!("Failed to revoke user roles: {}", res.text().await?)
        }

        Ok(())
    }

    pub async fn get_client_roles(
        token: &JwtTokens,
        search: Option<&str>,
    ) -> Result<Vec<KeycloakRole>> {
        let client = reqwest::Client::new();
        let base_url = CONFIG.keycloak_url();
        let realm_id = CONFIG.keycloak_realm();
        let client_uuid = CONFIG.keycloak_client_uuid();

        let mut url = format!("{base_url}/admin/realms/{realm_id}/clients/{client_uuid}/roles");
        if let Some(search) = search {
            url.push_str(&format!("?search={}", search));
        }

        let res = client
            .get(&url)
            .bearer_auth(token.access_token())
            .send()
            .await?;

        let roles = res.json::<Vec<KeycloakRole>>().await?;
        Ok(roles)
    }

    pub async fn get_client_role_by_name(
        token: &JwtTokens,
        role_name: &str,
    ) -> Result<KeycloakRole> {
        let client = reqwest::Client::new();
        let base_url = CONFIG.keycloak_url();
        let realm_id = CONFIG.keycloak_realm();
        let client_uuid = CONFIG.keycloak_client_uuid();

        let url =
            format!("{base_url}/admin/realms/{realm_id}/clients/{client_uuid}/roles/{role_name}");
        let res = client
            .get(&url)
            .bearer_auth(token.access_token())
            .send()
            .await?;

        let role = res.json::<KeycloakRole>().await?;

        Ok(role)
    }

    pub async fn create_client_role(tokens: &JwtTokens, role: &CreateRole) -> Result<()> {
        let client = reqwest::Client::new();
        let base_url = CONFIG.keycloak_url();
        let realm_id = CONFIG.keycloak_realm();
        let client_uuid = CONFIG.keycloak_client_uuid();
        let url = format!("{base_url}/admin/realms/{realm_id}/clients/{client_uuid}/roles");

        client
            .post(&url)
            .bearer_auth(tokens.access_token())
            .json(&role)
            .send()
            .await?;

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
        let base_url = CONFIG.keycloak_url();
        let realm_id = CONFIG.keycloak_realm();
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
            .field("cache_locked", &self.jwk_cache.try_lock().is_err())
            .finish()
    }
}

// Implement Clone manually
impl Clone for Keycloak {
    fn clone(&self) -> Self {
        Keycloak {
            jwk_cache: self.jwk_cache.clone(),
        }
    }
}
