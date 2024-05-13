use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::{
    constants::SYSTEM_PROMPT,
    enums::{
        OpenaiFinishReason, OpenaiMessageRole, OpenaiModel, OpenaiToolChoice,
        OpenaiToolFunctionParameterProperties, OpenaiToolName, OpenaiToolType,
    },
};

#[derive(Debug, Clone, Serialize)]
pub struct OpenaiCompletionRequest {
    model: OpenaiModel,
    messages: Vec<OpenaiMessage>,
    tools: Vec<OpenaiTool>,
    tool_choice: OpenaiToolChoice,
    stream: bool,
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

impl Default for OpenaiCompletionRequest {
    fn default() -> Self {
        let tools = vec![OpenaiTool::get(OpenaiToolName::SimilaritySearch)];

        let messages = vec![OpenaiMessage {
            role: OpenaiMessageRole::System,
            content: Some(SYSTEM_PROMPT.trim().to_owned()),
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
    content: Option<String>,
    tool_calls: Option<Vec<OpenaiToolCall>>,
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
                    parameters: None,
                },
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenaiToolFunction {
    name: OpenaiToolName,
    description: String,
    parameters: Option<OpenaiToolFunctionParameters>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenaiToolFunctionParameters {
    #[serde(rename = "type")]
    parameter_type: String,
    properties: OpenaiToolFunctionParameterProperties,
    required: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenaiStreamOptions {
    include_usage: bool,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct OpenAiStreamOutput {
    id: String,
    object: String,
    created: u64,
    model: String,
    system_fingerprint: Option<String>,
    choices: Vec<OpenaiChoice>,
    usage: Option<OpenaiUsage>,
}

impl OpenAiStreamOutput {
    pub fn from_chunk(s: &str) -> Result<Self> {
        let s = s.trim_start_matches("data: ").trim_end_matches("\n\n");

        match serde_json::from_str(s).map_err(Into::into) {
            Ok(output) => Ok(output),
            Err(e) => {
                let err = format!("{:?}", e);
                if !err.contains("missing field `name` at line 1 column 251") {
                    tracing::error!("Error: {:?}", e);
                    tracing::debug!("Error: {:?}", s);
                }
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
    id: String,
    #[serde(rename = "type")]
    tool_type: OpenaiToolType,
    function: OpenaiToolFunctionCall,
}

impl OpenaiToolCall {
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn function(&self) -> &OpenaiToolFunctionCall {
        &self.function
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenaiToolFunctionCall {
    name: OpenaiToolName,
    arguments: String,
}

impl OpenaiToolFunctionCall {
    pub fn name(&self) -> &OpenaiToolName {
        &self.name
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct OpenaiUsage {
    completion_tokens: u64,
    prompt_tokens: u64,
    tool_tokens: u64,
}

#[derive(Debug, Deserialize)]
pub struct OpenaiError {
    pub error: OpenaiErrorMessage,
}

#[derive(Debug, Deserialize)]
pub struct OpenaiErrorMessage {
    pub message: String,
}
