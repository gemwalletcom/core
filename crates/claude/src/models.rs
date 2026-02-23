use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct Request {
    pub(crate) model: String,
    pub(crate) max_tokens: u32,
    pub(crate) system: String,
    pub(crate) messages: Vec<Message>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Response {
    pub(crate) content: Vec<ContentBlock>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ContentBlock {
    pub(crate) text: Option<String>,
}
