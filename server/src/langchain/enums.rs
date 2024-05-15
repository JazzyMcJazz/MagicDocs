use serde::{Deserialize, Serialize};

pub enum LLMProvider {
    OpenAI,
}

pub enum LLMOutput {
    Content(String),
}

#[derive(Debug, Clone)]
pub enum OpenaiEmbeddingInput {
    String(String),
    StringArray(Vec<String>),
}

impl Serialize for OpenaiEmbeddingInput {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            OpenaiEmbeddingInput::String(s) => serializer.serialize_str(s),
            OpenaiEmbeddingInput::StringArray(v) => v.serialize(serializer),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OpenaiEncodingFormat {
    Float,
}

#[derive(Debug, Clone, Default, Serialize)]
#[allow(dead_code)]
pub enum OpenaiModel {
    #[serde(rename = "gpt-3.5-turbo")]
    GPT35Turbo,
    #[serde(rename = "gpt-4-turbo")]
    GPT4Turbo,
    #[default]
    #[serde(rename = "gpt-4o")]
    GPT4o,
    #[serde(rename = "text-embedding-3-small")]
    TextEmbedding3Small,
}

#[derive(Debug, Clone, Serialize)]
pub enum OpenaiMessageRole {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(rename = "tool")]
    Tool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default, Serialize)]
pub enum OpenaiToolChoice {
    #[serde(rename = "none")]
    None,
    #[default]
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "required")]
    Required,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpenaiToolName {
    #[serde(rename = "similarity_search")]
    SimilaritySearch,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum OpenaiToolType {
    #[default]
    #[serde(rename = "function")]
    Function,
}

#[derive(Debug, Clone, Deserialize)]
pub enum OpenaiFinishReason {
    #[serde(rename = "stop")]
    Stop,
    #[serde(rename = "length")]
    Length,
    #[serde(rename = "content_filter")]
    ContentFilter,
    #[serde(rename = "tool_calls")]
    ToolCalls,
}
