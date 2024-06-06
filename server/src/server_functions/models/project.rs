use leptos::server_fn::serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectData {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub version: i32,
    pub versions: Vec<i32>,
    pub finalized: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectPermissions {
    pub read: bool,
    pub write: bool,
    pub delete: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectDocument {
    pub id: i32,
    pub name: String,
    pub is_embedded: bool,
}
