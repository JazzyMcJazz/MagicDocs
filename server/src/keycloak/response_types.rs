use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct TokenResponse {
    access_token: String,
    refresh_token: String,
    id_token: String,
    expires_in: i64,
}

impl TokenResponse {
    pub fn access_token(&self) -> &String {
        &self.access_token
    }
    pub fn refresh_token(&self) -> &String {
        &self.refresh_token
    }
    pub fn id_token(&self) -> &String {
        &self.id_token
    }
    pub fn expires_in(&self) -> i64 {
        self.expires_in
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeycloakRole {
    id: String,
    name: String,
    description: Option<String>,
}

impl KeycloakRole {
    pub fn new(id: String, name: String, description: Option<String>) -> Self {
        Self {
            id,
            name,
            description,
        }
    }
    pub fn id(&self) -> &String {
        &self.id
    }
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn description(&self) -> Option<String> {
        self.description.to_owned()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeycloakRoleMapping {
    #[serde(rename = "clientMappings")]
    client_mappings: Option<HashMap<String, KeycloakRoleMappingMappings>>,
}

impl KeycloakRoleMapping {
    pub fn client_roles(&self, client: &str) -> Option<Vec<KeycloakRole>> {
        self.client_mappings
            .to_owned()?
            .get(client)
            .map(|mappings| mappings.mappings.to_owned())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeycloakRoleMappingMappings {
    mappings: Vec<KeycloakRole>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeycloakUser {
    #[serde(rename = "createdTimestamp")]
    created_timestamp: i128,
    email: String,
    #[serde(rename = "emailVerified")]
    email_verified: bool,
    enabled: bool,
    #[serde(rename = "firstName")]
    first_name: String,
    id: String,
    #[serde(rename = "lastName")]
    last_name: String,
    username: String,
}

impl KeycloakUser {
    pub fn created_timestamp(&self) -> i128 {
        self.created_timestamp
    }
    pub fn email(&self) -> &String {
        &self.email
    }
    pub fn email_verified(&self) -> bool {
        self.email_verified
    }
    pub fn enabled(&self) -> bool {
        self.enabled
    }
    pub fn first_name(&self) -> &String {
        &self.first_name
    }
    pub fn id(&self) -> &String {
        &self.id
    }
    pub fn last_name(&self) -> &String {
        &self.last_name
    }
    pub fn username(&self) -> &String {
        &self.username
    }
}
