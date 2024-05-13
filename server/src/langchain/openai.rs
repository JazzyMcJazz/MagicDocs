use std::collections::VecDeque;

use anyhow::{bail, Result};
use futures_util::{stream::Stream, StreamExt};
use llm_chain::traits::Embeddings;
use llm_chain_openai::embeddings;
use migration::sea_orm::DatabaseConnection;
use text_splitter::{ChunkConfig, MarkdownSplitter};
use tiktoken_rs::cl100k_base;
use tokio::pin;

use crate::{
    langchain::{
        enums::{OpenaiFinishReason::*, OpenaiToolName::*},
        models::OpenAiStreamOutput,
    },
    models::Embedding,
};

use super::{
    enums::LLMOutput,
    models::{OpenaiCompletionRequest, OpenaiError, OpenaiToolCall},
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

    pub fn event_loop<'a>(
        db: &'a DatabaseConnection,
        prompt: &'a str,
    ) -> Result<impl Stream<Item = Result<LLMOutput>> + 'a> {
        let stream = async_stream::stream! {
            let mut request = OpenaiCompletionRequest::default();
            request.add_user_msg(prompt);

            'request_loop: loop {
                let response = Self::completion_stream(&request).await?;
                pin!(response);

                let mut tool_calls = Vec::new();

                while let Some(output) = response.next().await {
                    match output {
                        Ok(output) => {

                            if output.usage().is_some() {
                                tracing::info!("Usage: {:?}", output.usage());
                            }

                            for choice in output.choices() {
                                if let Some(finish_reason) = choice.finish_reason() {
                                    match finish_reason {
                                        ToolCalls => {
                                            request = Self::handle_tool_calls(db, &tool_calls, &mut request).await?;
                                            continue 'request_loop;
                                        },
                                        _ => {
                                            tracing::info!("Finish reason: {:?}", finish_reason);
                                            break 'request_loop;
                                        }
                                    }
                                }

                                if let Some(delta) = choice.delta() {
                                    if let Some(content) = delta.content() {
                                        yield Ok(LLMOutput::Content(content.to_owned()));
                                    }
                                    if let Some(calls) = delta.tool_calls() {
                                        tool_calls.extend(calls.to_owned());
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Error: {:?}", e);
                            break;
                            // yield Err(e);
                        }
                    }
                }
                tracing::warn!("Warning: Reached end of event loop without break or continue!");
                break;
            }
        };

        Ok(stream)
    }

    pub async fn handle_tool_calls(
        _db: &DatabaseConnection,
        tools: &Vec<OpenaiToolCall>,
        request: &mut OpenaiCompletionRequest,
    ) -> Result<OpenaiCompletionRequest> {
        request.add_tool_calls(tools.to_owned());
        let Some(tool) = tools.first() else {
            tracing::error!("No tool found in tool calls");
            bail!("No tool found in tool calls");
        };

        match tool.function().name() {
            SimilaritySearch => {
                request.add_tool_result("No results found", tool.id());
                request.disable_tools();
                // let embeddings = Self::embed(prompt).await.unwrap();
            }
        }

        Ok(request.to_owned())
    }

    pub async fn completion_stream(
        json: &OpenaiCompletionRequest,
    ) -> Result<impl Stream<Item = Result<OpenAiStreamOutput>>> {
        let client = reqwest::Client::new();

        let api_key = std::env::var("OPENAI_API_KEY")?;

        let mut res = client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(api_key)
            .json(&json)
            .send()
            .await?;

        let stream = async_stream::stream! {
            let mut buffer = String::new();

            while let Some(chunk) = res.chunk().await? {
                let s = String::from_utf8_lossy(&chunk);
                if s.starts_with("{\n  \"error\":") {
                    let err = serde_json::from_str::<OpenaiError>(&s)?;
                    tracing::error!("Openai Response Error: {:?}", err.error.message);
                    break;
                }

                buffer.push_str(&s);

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
                    if message.is_empty() {
                        continue;
                    }

                    if let Ok(output) = OpenAiStreamOutput::from_chunk(message) {
                        yield Ok(output);
                    }
                }
            }
        };

        Ok(stream)
    }
}
