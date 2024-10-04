use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityResponse {
    pub code: i32,
    pub message: String,
    pub result: Option<Value>,
}
