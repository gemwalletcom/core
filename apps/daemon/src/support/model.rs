use serde::{Deserialize, Serialize};

pub const EVENT_MESSAGE_CREATED: &str = "message_created";
pub const EVENT_CONVERSATION_UPDATED: &str = "conversation_updated";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatwootWebhookPayload {
    pub event: String,
    pub message_type: Option<String>,
    pub conversation: Option<Conversation>,
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub meta: Meta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    pub sender: Sender,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomAttributes {
    #[serde(rename = "deviceId")]
    pub device_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sender {
    pub custom_attributes: Option<CustomAttributes>,
}

impl ChatwootWebhookPayload {
    pub fn get_device_id(&self) -> Option<String> {
        self.conversation
            .as_ref()
            .and_then(|conversation| conversation.meta.sender.custom_attributes.as_ref().and_then(|attrs| attrs.device_id.clone()))
    }
}
