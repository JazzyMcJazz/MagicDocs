use leptos::server_fn::serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: i32,
    pub name: String,
    pub content: String,
    pub source: Option<String>,
}
