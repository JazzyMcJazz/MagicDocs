use std::{fmt, sync::Arc};
use tokio::sync::Mutex;

use keycloak::{KeycloakAdmin, KeycloakAdminToken};

pub struct Keycloak {
    admin: Arc<Mutex<KeycloakAdmin>>,
    base_url: String,
    realm_id: String,
    client_id: String,
    client_secret: String,
}

impl Keycloak {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let base_url =
            std::env::var("KEYCLOAK_ADDR").unwrap_or_else(|_| "http://localhost:8080".into());
        let user = std::env::var("KEYCLOAK_USER").unwrap_or_else(|_| "admin".into());
        let password = std::env::var("KEYCLOAK_PASSWORD").unwrap_or_else(|_| "password".into());
        let realm_id = std::env::var("KEYCLOAK_REALM").unwrap_or_else(|_| "magicdocs".into());
        let client_id = std::env::var("KEYCLOAK_CLIENT").unwrap_or_else(|_| "magicdocs".into());
        let client_secret = std::env::var("KEYCLOAK_CLIENT_SECRET").unwrap_or_else(|_| "".into());
        let reqwest_client = reqwest::Client::new();
        let admin_token =
            KeycloakAdminToken::acquire(&base_url, &user, &password, &reqwest_client).await?;

        let admin = KeycloakAdmin::new(&base_url, admin_token, reqwest_client);

        Ok(Self {
            admin: Arc::new(Mutex::new(admin)),
            base_url,
            realm_id,
            client_id,
            client_secret,
        })
    }

    pub async fn exchange_code(
        &self,
        code: &str,
        redirect_uri: &str,
    ) -> Result<super::TokenResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let base_url = &self.base_url;
        let realm_id = &self.realm_id;
        let client_secret = &self.client_secret;
        let token_url = format!("{base_url}/realms/{realm_id}/protocol/openid-connect/token");
        let params = [
            ("grant_type", "authorization_code"),
            ("client_id", &self.client_id),
            ("redirect_uri", redirect_uri),
            ("code", code),
            ("client_secret", client_secret),
        ];

        let response = client.post(&token_url).form(&params).send().await;

        match response {
            Ok(response) => {
                if response.status().is_success() {
                    let token = response.json::<super::TokenResponse>().await?;
                    dbg!(&token);
                    Ok(token)
                } else {
                    dbg!(response.text().await?);
                    Err("Failed to exchange code for token".into())
                }
            }
            Err(e) => Err(e.into()),
        }
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
}

impl fmt::Debug for Keycloak {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Keycloak")
            .field("base_url", &self.base_url)
            .field("realm_id", &self.realm_id)
            .field("client_id", &self.client_id)
            // Optionally show whether admin is currently locked
            .field("admin_locked", &self.admin.try_lock().is_err())
            .finish()
    }
}

// Implement Clone manually
impl Clone for Keycloak {
    fn clone(&self) -> Self {
        Keycloak {
            admin: self.admin.clone(), // Clones the Arc, not the KeycloakAdmin
            base_url: self.base_url.clone(),
            realm_id: self.realm_id.clone(),
            client_id: self.client_id.clone(),
            client_secret: self.client_secret.clone(),
        }
    }
}
