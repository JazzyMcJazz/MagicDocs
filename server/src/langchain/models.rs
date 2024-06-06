use anyhow::Result;
use entity::project;
use serde::{Deserialize, Serialize};

use super::{
    constants::SYSTEM_PROMPT,
    enums::{
        OpenaiEmbeddingInput, OpenaiEncodingFormat, OpenaiFinishReason, OpenaiMessageRole,
        OpenaiModel, OpenaiToolChoice, OpenaiToolName, OpenaiToolType,
    },
};

#[derive(Debug, Clone, Serialize)]
pub struct OpenaiEmbeddingRequest {
    input: OpenaiEmbeddingInput,
    model: OpenaiModel,
    encoding_format: OpenaiEncodingFormat,
}

impl OpenaiEmbeddingRequest {
    pub fn new(input: OpenaiEmbeddingInput) -> Self {
        Self {
            input,
            model: OpenaiModel::TextEmbedding3Small,
            encoding_format: OpenaiEncodingFormat::Float,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenaiEmbeddingResponse {
    data: Vec<OpenaiEmbeddingData>,
}

impl OpenaiEmbeddingResponse {
    pub fn data(&self) -> &Vec<OpenaiEmbeddingData> {
        &self.data
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenaiEmbeddingData {
    index: u64,
    embedding: Vec<f32>,
}

impl OpenaiEmbeddingData {
    pub fn index(&self) -> u64 {
        self.index
    }
    pub fn embedding(&self) -> &Vec<f32> {
        &self.embedding
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenaiCompletionRequest {
    model: OpenaiModel,
    messages: Vec<OpenaiMessage>,
    tools: Vec<OpenaiTool>,
    tool_choice: OpenaiToolChoice,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream_options: Option<OpenaiStreamOptions>,
}

impl OpenaiCompletionRequest {
    pub fn add_user_msg(&mut self, content: &str) {
        self.messages.push(OpenaiMessage {
            role: OpenaiMessageRole::User,
            content: Some(content.to_owned()),
            tool_calls: None,
            tool_call_id: None,
        });
    }
    pub fn add_tool_calls(&mut self, tool_calls: Vec<OpenaiToolCall>) {
        self.messages.push(OpenaiMessage {
            role: OpenaiMessageRole::Assistant,
            content: None,
            tool_calls: Some(tool_calls),
            tool_call_id: None,
        });
    }

    pub fn add_tool_result(&mut self, content: &str, tool_call_id: &str) {
        self.messages.push(OpenaiMessage {
            role: OpenaiMessageRole::Tool,
            content: Some(content.to_owned()),
            tool_calls: None,
            tool_call_id: Some(tool_call_id.to_owned()),
        });
    }

    pub fn disable_tools(&mut self) {
        self.tool_choice = OpenaiToolChoice::None;
    }
}

impl OpenaiCompletionRequest {
    pub fn new(project: project::Model, version: i32) -> Self {
        let tools = vec![OpenaiTool::get(OpenaiToolName::SimilaritySearch)];

        let system_prompt = SYSTEM_PROMPT
            .replace("{{ name }}", &project.name)
            .replace("{{ version }}", &version.to_string())
            .replace(
                "{{ description }}",
                if project.description.is_empty() {
                    "None"
                } else {
                    &project.description
                },
            );

        let messages = vec![OpenaiMessage {
            role: OpenaiMessageRole::System,
            content: Some(system_prompt),
            tool_call_id: None,
            tool_calls: None,
        }];

        let options = OpenaiStreamOptions {
            include_usage: true,
        };

        Self {
            model: OpenaiModel::default(),
            messages,
            tools,
            stream: true,
            stream_options: Some(options),
            tool_choice: OpenaiToolChoice::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenaiMessage {
    role: OpenaiMessageRole,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OpenaiToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenaiTool {
    #[serde(rename = "type")]
    tool_type: OpenaiToolType,
    function: OpenaiToolFunction,
}

impl OpenaiTool {
    fn get(name: OpenaiToolName) -> Self {
        match name {
            OpenaiToolName::SimilaritySearch => OpenaiTool {
                tool_type: OpenaiToolType::Function,
                function: OpenaiToolFunction {
                    name: OpenaiToolName::SimilaritySearch,
                    description: "Search embedded documents for relevant information to the query"
                        .to_owned(),
                    parameters: Some(OpenaiToolFunctionParameters {
                        parameter_type: "object".to_owned(),
                        properties: serde_json::json!({
                            "query": {
                                "type": "string",
                                "description": "The query to search for in the embedded documents. The query is generated from the user's input into a query that is more likely to return relevant information from the database when cosine distance is invoked."
                            }
                        }),
                        required: vec!["query".to_owned()],
                    }),
                },
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenaiToolFunction {
    name: OpenaiToolName,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<OpenaiToolFunctionParameters>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenaiToolFunctionParameters {
    #[serde(rename = "type")]
    parameter_type: String,
    properties: serde_json::Value,
    required: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenaiStreamOptions {
    include_usage: bool,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct OpenaiStreamOutput {
    id: String,
    object: String,
    created: u64,
    model: String,
    system_fingerprint: Option<String>,
    choices: Vec<OpenaiChoice>,
    usage: Option<OpenaiUsage>,
}

impl OpenaiStreamOutput {
    pub fn from_chunk(s: &str) -> Result<Self> {
        match serde_json::from_str(s).map_err(Into::into) {
            Ok(output) => Ok(output),
            Err(e) => {
                tracing::error!("Error: {:?}", e);
                tracing::debug!("Error: {:?}", s);
                Err(e)
            }
        }
    }

    pub fn choices(&self) -> &Vec<OpenaiChoice> {
        &self.choices
    }

    pub fn usage(&self) -> Option<OpenaiUsage> {
        self.usage.to_owned()
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct OpenaiChoice {
    index: u64,
    delta: Option<OpenaiDelta>,
    finish_reason: Option<OpenaiFinishReason>,
}

impl OpenaiChoice {
    pub fn delta(&self) -> Option<&OpenaiDelta> {
        self.delta.as_ref()
    }
    pub fn finish_reason(&self) -> Option<OpenaiFinishReason> {
        self.finish_reason.to_owned()
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct OpenaiDelta {
    content: Option<String>,
    tool_calls: Option<Vec<OpenaiToolCall>>,
    role: Option<String>,
}

impl OpenaiDelta {
    pub fn content(&self) -> Option<&String> {
        self.content.as_ref()
    }

    pub fn tool_calls(&self) -> Option<&Vec<OpenaiToolCall>> {
        self.tool_calls.as_ref()
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenaiToolCall {
    index: u64,
    id: Option<String>,
    #[serde(rename = "type")]
    tool_type: Option<OpenaiToolType>,
    function: OpenaiToolFunctionCall,
}

impl OpenaiToolCall {
    pub fn index(&self) -> u64 {
        self.index
    }
    pub fn id(&self) -> &Option<String> {
        &self.id
    }
    pub fn function(&self) -> &OpenaiToolFunctionCall {
        &self.function
    }
    pub fn update_function(&mut self, argument: &str) {
        self.function.push_argument(argument);
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenaiToolFunctionCall {
    name: Option<OpenaiToolName>,
    arguments: String,
}

impl OpenaiToolFunctionCall {
    pub fn name(&self) -> &Option<OpenaiToolName> {
        &self.name
    }
    pub fn arguments(&self) -> &str {
        &self.arguments
    }
    pub fn push_argument(&mut self, arg: &str) {
        self.arguments.push_str(arg);
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct OpenaiUsage {
    completion_tokens: u64,
    prompt_tokens: u64,
    total_tokens: u64,
}

#[derive(Debug, Deserialize)]
pub struct OpenaiError {
    pub error: OpenaiErrorMessage,
}

#[derive(Debug, Deserialize)]
pub struct OpenaiErrorMessage {
    pub message: String,
}
