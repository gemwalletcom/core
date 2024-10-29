use primitives::PushNotification;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone)]
pub struct Notification {
    pub tokens: Vec<String>,
    pub platform: i32,
    pub title: String,
    pub message: String,
    pub topic: Option<String>,
    pub data: PushNotification,
}

impl Notification {
    pub fn new(
        tokens: Vec<String>,
        platform: i32,
        title: String,
        message: String,
        data: PushNotification,
    ) -> Self {
        Self {
            tokens,
            platform,
            title,
            message,
            topic: None,
            data,
        }
    }

    pub fn with_topic(mut self, topic: Option<String>) -> Self {
        self.topic = topic;
        self
    }
}

#[derive(Debug, Serialize)]
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
