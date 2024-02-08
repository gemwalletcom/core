use primitives::PushNotification;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Notification {
    pub tokens: Vec<String>,
    pub platform: i32,
    pub title: String,
    pub message: String,
    pub topic: Option<String>,
    pub data: Option<PushNotification>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Notifications {
    pub notifications: Vec<Notification>,
}

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
