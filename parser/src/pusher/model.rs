use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Notification {
    pub tokens: Vec<String>,
    pub platform: i32,
    pub title: String,
    pub message: String,
    pub topic: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Notifications {
    pub notifications: Vec<Notification>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub counts: i32,
    pub logs: Vec<String>,
    pub success: String,
}