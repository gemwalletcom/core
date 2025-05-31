use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub counts: i32,
    pub logs: Vec<Log>,
    pub success: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Log {
    #[serde(rename = "type")]
    pub log_type: String,
    pub platform: String,
    pub token: String,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub title: String,
    pub message: Option<String>,
}
