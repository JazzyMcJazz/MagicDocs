use std::collections::VecDeque;

use anyhow::{anyhow, bail, Result};
use futures_util::{stream::Stream, StreamExt};
use migration::sea_orm::DatabaseConnection;
use serde::Deserialize;
use text_splitter::{ChunkConfig, MarkdownSplitter};
use tiktoken_rs::cl100k_base;
use tokio::pin;

use crate::{
    database::Repo,
    langchain::{
        enums::{OpenaiFinishReason::*, OpenaiToolName::*},
        models::OpenaiStreamOutput,
    },
    models::Embedding,
    utils::config::Config,
};

use super::{
    enums::{LLMOutput, OpenaiEmbeddingInput},
    models::{
        OpenaiCompletionRequest, OpenaiEmbeddingRequest, OpenaiEmbeddingResponse, OpenaiError,
        OpenaiToolCall,
    },
};

pub struct OpenAI;

impl OpenAI {
    pub async fn embed_document(content: &str) -> Result<Vec<Embedding>> {
        let config = Config::default();
        let client = reqwest::Client::new();

        // Split content into chunks
        let tokenizer = cl100k_base()?;
        let max_characters = 500..2000;
        let chunk_config = ChunkConfig::new(max_characters).with_sizer(tokenizer);
        let splitter = MarkdownSplitter::new(chunk_config);
        let chunks = splitter.chunks(content);
        let texts = chunks
            .into_iter()
            .map(|chunk| chunk.to_owned())
            .collect::<Vec<_>>();

        // Send embedding request
        let input = OpenaiEmbeddingInput::StringArray(texts.to_owned());
        let json = OpenaiEmbeddingRequest::new(input);

        let res = client
            .post("https://api.openai.com/v1/embeddings")
            .bearer_auth(config.openai_api_key())
            .json(&json)
            .send()
            .await?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await?;
            tracing::error!("Embedding request failed with message: {:?}", text);
            bail!("Embedding request failed with status: {:?}", status);
        }

        // Parse response
        let data = res.json::<OpenaiEmbeddingResponse>().await?;

        // Sort by index (order of appearance in the text)
        let mut data = data.data().to_vec();
        data.sort_by_key(|d| d.index());

        let embedded_vecs = data
            .iter()
            .map(|d| d.embedding().to_owned())
            .collect::<Vec<_>>();
        let embeddings = texts
            .into_iter()
            .zip(embedded_vecs.into_iter())
            .map(|(text, vec)| Embedding::new(text, vec))
            .collect::<Vec<_>>();

        Ok(embeddings)
    }

    pub async fn embed_query(query: &str) -> Result<Vec<f32>> {
        let config = Config::default();
        let client = reqwest::Client::new();

        let input = OpenaiEmbeddingInput::String(query.to_owned());
        let json = OpenaiEmbeddingRequest::new(input);
        let res = client
            .post("https://api.openai.com/v1/embeddings")
            .bearer_auth(config.openai_api_key())
            .json(&json)
            .send()
            .await?;

        if !res.status().is_success() {
            tracing::error!("Embedding request failed with status: {:?}", res.status());
            bail!("Embedding request failed with status: {:?}", res.status());
        }

        let data = res.json::<OpenaiEmbeddingResponse>().await?;
        let vec = data
            .data()
            .first()
            .ok_or_else(|| anyhow!("No data found in response"))?
            .embedding()
            .to_owned();

        Ok(vec)
    }

    pub fn event_loop<'a>(
        db: &'a DatabaseConnection,
        project_id: i32,
        version: i32,
        prompt: &'a str,
    ) -> Result<impl Stream<Item = Result<LLMOutput>> + 'a> {
        let stream = async_stream::stream! {
            let mut request = OpenaiCompletionRequest::default();
            request.add_user_msg(prompt);

            'request_loop: loop {
                let response = Self::completion_stream(&request).await?;
                pin!(response);

                let mut tool_calls: Vec<OpenaiToolCall> = Vec::new();
                let mut finish_reason = None;

                // Handle response
                while let Some(output) = response.next().await {
                    match output {
                        Ok(output) => {

                            if output.usage().is_some() {
                                tracing::info!("Usage: {:?}", output.usage());
                            }

                            let Some(choice) = output.choices().first() else {
                                continue;
                            };

                            if let Some(reason) = choice.finish_reason() {
                                finish_reason = Some(reason);
                            }

                            if let Some(delta) = choice.delta() {
                                if let Some(content) = delta.content() {
                                    yield Ok(LLMOutput::Content(content.to_owned()));
                                }
                                if let Some(calls) = delta.tool_calls() {
                                    for call in calls {
                                        let existing = tool_calls.iter_mut().find(|c| c.index() == call.index());
                                        match existing {
                                            // Update existing tool call
                                            Some(existing) => {
                                                existing.update_function(call.function().arguments());
                                            },
                                            // Add new tool call
                                            None => {
                                                tool_calls.push(call.to_owned());
                                            }
                                        }
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

                let finish_reason = finish_reason.ok_or_else(|| anyhow!("No finish reason found"))?;

                // Process tool calls
                match finish_reason {
                    ToolCalls => {
                        request = Self::handle_tool_calls(db, project_id, version, &tool_calls, &mut request).await?;
                        continue 'request_loop;
                    },
                    _ => {
                        tracing::info!("Finish reason: {:?}", finish_reason);
                        break 'request_loop;
                    }
                }
            }
        };

        Ok(stream)
    }

    pub async fn handle_tool_calls(
        db: &DatabaseConnection,
        project_id: i32,
        version: i32,
        tools: &Vec<OpenaiToolCall>,
        request: &mut OpenaiCompletionRequest,
    ) -> Result<OpenaiCompletionRequest> {
        request.add_tool_calls(tools.to_owned());
        let Some(tool) = tools.first() else {
            tracing::error!("No tool found in tool calls");
            bail!("No tool found in tool calls");
        };

        let Some(function_name) = tool.function().name() else {
            bail!("No function name found in tool");
        };

        dbg!(&tool);

        match function_name {
            SimilaritySearch => {
                #[derive(Debug, Deserialize)]
                struct Query {
                    query: String,
                }
                if let Some(id) = tool.id() {
                    let query = serde_json::from_str::<Query>(tool.function().arguments())?;
                    let embedded_query = Self::embed_query(&query.query).await?;
                    let result = db
                        .embeddings()
                        .similarity_search(project_id, version, embedded_query)
                        .await?;
                    let content = result
                        .iter()
                        .map(|r| r.text.to_owned())
                        .collect::<Vec<_>>()
                        .join("\n");

                    request.add_tool_result(&content, id);
                }
                request.disable_tools();
            }
        }

        Ok(request.to_owned())
    }

    pub async fn completion_stream(
        json: &OpenaiCompletionRequest,
    ) -> Result<impl Stream<Item = Result<OpenaiStreamOutput>>> {
        let config = Config::default();
        let client = reqwest::Client::new();

        // dbg!(&json);
        let mut res = client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(config.openai_api_key())
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
                    if message.is_empty() || message.eq("[DONE]") {
                        continue;
                    }

                    if let Ok(output) = OpenaiStreamOutput::from_chunk(message) {
                        yield Ok(output);
                    }
                }
            }
        };

        Ok(stream)
    }
}
