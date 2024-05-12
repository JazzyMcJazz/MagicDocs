use std::collections::VecDeque;

use anyhow::{bail, Result};
use futures_util::stream;
use llm_chain::traits::Embeddings;
use llm_chain_openai::embeddings;
use serde_json::json;
use text_splitter::{ChunkConfig, MarkdownSplitter};
use tiktoken_rs::cl100k_base;

use crate::{
    langchain::{constants::SYSTEM_PROMPT, models::OpenAiStreamOutput},
    models::Embedding,
};

pub struct OpenAI;

impl OpenAI {
    pub async fn embed(content: &str) -> Result<Vec<Embedding>> {
        let embeddings = embeddings::Embeddings::default();

        let tokenizer = cl100k_base()?;
        let max_characters = 500..2000;
        let chunk_config = ChunkConfig::new(max_characters).with_sizer(tokenizer);
        let splitter = MarkdownSplitter::new(chunk_config);

        let chunks = splitter.chunks(content);
        let texts = chunks
            .into_iter()
            .map(|chunk| chunk.to_owned())
            .collect::<Vec<_>>();

        let embedded_vecs = match embeddings.embed_texts(texts.to_owned()).await {
            Ok(embedded_vecs) => embedded_vecs,
            Err(e) => {
                tracing::error!("Failed to embed texts: {:?}", e);
                bail!("Failed to embed texts: {:?}", e);
            }
        };

        let embeddings = texts
            .into_iter()
            .zip(embedded_vecs.into_iter())
            .map(|(text, vec)| Embedding::new(text, vec))
            .collect::<Vec<_>>();

        Ok(embeddings)
    }

    pub async fn completion(
        prompt: &str,
    ) -> Result<impl stream::Stream<Item = Result<OpenAiStreamOutput>>> {
        let client = reqwest::Client::new();

        let api_key = std::env::var("OPENAI_API_KEY")?;

        let mut res = client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(api_key)
            .json(&json!({
                "model": "gpt-4-turbo",
                "messages": [
                    {
                        "role": "system",
                        "content": SYSTEM_PROMPT,
                    },
                    {
                        "role": "user",
                        "content": prompt,
                    },
                ],
                "stream": true,
            }))
            .send()
            .await?;

        let stream = async_stream::stream! {
            let mut buffer = String::new();

            while let Some(chunk) = res.chunk().await? {
                buffer.push_str(&String::from_utf8_lossy(&chunk));

                let mut messages = VecDeque::new();
                let mut temp_buffer = String::new();

                let buffer_clone = buffer.clone();
                let parts: Vec<&str> = buffer_clone.split("\n\n").collect();
                for (i, part) in parts.iter().enumerate() {
                    if i == parts.len() - 1 && !part.is_empty() {
                        temp_buffer.push_str(part);
                    } else {
                        messages.push_back(part.trim_start_matches("data: "));
                    }
                }

                buffer = temp_buffer;

                for message in messages {
                    if let Ok(output) = OpenAiStreamOutput::from_chunk(message) {
                        yield Ok(output);
                    }
                }
            }
        };

        Ok(stream)
    }
}
