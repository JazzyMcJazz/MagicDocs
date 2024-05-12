use crate::models::Embedding;
use anyhow::Result;
use futures_util::Stream;

pub use self::enums::LLMProvider;
use self::{models::OpenAiStreamOutput, openai::OpenAI};

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
            LLMProvider::OpenAI => OpenAI::embed(content).await,
        }
    }

    pub async fn chat_completion(
        &self,
        prompt: &str,
    ) -> Result<impl Stream<Item = Result<OpenAiStreamOutput>>> {
        match self.0 {
            LLMProvider::OpenAI => OpenAI::completion(prompt).await,
        }
    }
}
