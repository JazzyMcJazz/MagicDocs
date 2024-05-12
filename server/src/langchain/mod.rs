use crate::models::Embedding;
use anyhow::Result;

mod embedding;

pub struct Langchain;

impl Langchain {
    pub async fn embed(content: &str) -> Result<Vec<Embedding>> {
        embedding::embed(content).await
    }
}
