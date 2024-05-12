use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ChatForm {
    pub message: String,
}
