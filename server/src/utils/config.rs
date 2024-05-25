pub struct Config {
    rust_env: String,
    my_log: String,
    // rust_log: String,
    database_url: String,
    keycloak_url: String,
    keycloak_realm: String,
    keycloak_client_name: String,
    keycloak_client_uuid: String,
    keycloak_client_secret: String,
    openai_api_key: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rust_env: std::env::var("RUST_ENV").unwrap_or("prod".to_string()),
            my_log: std::env::var("MY_LOG").unwrap_or("info".to_string()),
            database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            keycloak_url: std::env::var("KEYCLOAK_URL").expect("KEYCLOAK_URL must be set"),
            keycloak_realm: std::env::var("KEYCLOAK_REALM").expect("KEYCLOAK_REALM must be set"),
            keycloak_client_name: std::env::var("KEYCLOAK_CLIENT_NAME")
                .expect("KEYCLOAK_CLIENT_NAME must be set"),
            keycloak_client_uuid: std::env::var("KEYCLOAK_CLIENT_UUID")
                .expect("KEYCLOAK_CLIENT_UUID must be set"),
            keycloak_client_secret: std::env::var("KEYCLOAK_CLIENT_SECRET")
                .expect("KEYCLOAK_CLIENT_SECRET must be set"),
            openai_api_key: std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set"),
        }
    }
}

impl Config {
    pub fn rust_env(&self) -> &str {
        &self.rust_env
    }

    pub fn my_log(&self) -> &str {
        &self.my_log
    }

    pub fn database_url(&self) -> &str {
        &self.database_url
    }

    pub fn keycloak_url(&self) -> &str {
        &self.keycloak_url
    }

    pub fn keycloak_realm(&self) -> &str {
        &self.keycloak_realm
    }

    pub fn keycloak_client_name(&self) -> &str {
        &self.keycloak_client_name
    }

    pub fn keycloak_client_uuid(&self) -> &str {
        &self.keycloak_client_uuid
    }

    pub fn keycloak_client_secret(&self) -> &str {
        &self.keycloak_client_secret
    }

    pub fn openai_api_key(&self) -> &str {
        &self.openai_api_key
    }
}
