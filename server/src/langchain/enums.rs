use serde::{Deserialize, Serialize};

pub enum LLMProvider {
    OpenAI,
}

pub enum LLMOutput {
    Content(String),
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

#[derive(Debug, Clone, Serialize)]
pub enum OpenaiToolFunctionParameterProperties {}

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
