use anyhow::Result;
use serde::Deserialize;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct OpenAiStreamOutput {
    id: String,
    object: String,
    created: u64,
    model: String,
    system_fingerprint: Option<String>,
    choices: Vec<OpenAiChoice>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct OpenAiChoice {
    index: u64,
    delta: Option<OpenAiDelta>,
    finish_reason: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct OpenAiDelta {
    content: Option<String>,
}

impl OpenAiStreamOutput {
    pub fn from_chunk(s: &str) -> Result<Self> {
        let s = s.trim_start_matches("data: ").trim_end_matches("\n\n");

        serde_json::from_str(s).map_err(Into::into)
    }

    pub fn content(&self) -> Option<String> {
        let mut content = String::new();
        for choice in &self.choices {
            match &choice.delta {
                Some(delta) => {
                    if let Some(c) = &delta.content {
                        content.push_str(c);
                    }
                }
                None => return None,
            }
        }
        Some(content)
    }
}
