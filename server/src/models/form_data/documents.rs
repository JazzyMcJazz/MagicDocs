use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateDocumentForm {
    #[serde(rename = "title")]
    pub name: String,
    pub content: String,
}
