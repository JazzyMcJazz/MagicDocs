use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateDocumentForm {
    #[serde(rename = "title")]
    pub name: String,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StartCrawlerForm {
    pub pages: String,
    pub url: String,
}
