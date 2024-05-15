use crate::models::Embedding;
use anyhow::Result;
use futures_util::Stream;
use migration::sea_orm::DatabaseConnection;

pub use self::enums::{LLMOutput, LLMProvider};
use self::openai::OpenAI;

mod constants;
mod enums;
mod models;
mod openai;

pub struct Langchain(LLMProvider);

impl Langchain {
    pub fn new(provider: LLMProvider) -> Self {
        Self(provider)
    }

    pub async fn embed(&self, content: &str) -> Result<Vec<Embedding>> {
        match self.0 {
            LLMProvider::OpenAI => OpenAI::embed_document(content).await,
        }
    }

    pub fn chat_completion<'a>(
        &self,
        db: &'a DatabaseConnection,
        project_id: i32,
        version: i32,
        prompt: &'a str,
    ) -> Result<impl Stream<Item = Result<LLMOutput>> + 'a> {
        match self.0 {
            LLMProvider::OpenAI => OpenAI::event_loop(db, project_id, version, prompt),
        }
    }
}
