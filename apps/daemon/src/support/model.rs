use serde::{Deserialize, Serialize};

pub const EVENT_MESSAGE_CREATED: &str = "message_created";
pub const EVENT_CONVERSATION_UPDATED: &str = "conversation_updated";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatwootWebhookPayload {
    pub event: String,
    pub message_type: Option<String>,
    pub meta: Option<MetaSender>,
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaSender {
    pub sender: Sender,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomAttributes {
    #[serde(rename = "deviceId")]
    pub device_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sender {
    pub name: String,
    pub custom_attributes: Option<CustomAttributes>,
}

impl ChatwootWebhookPayload {
    pub fn get_device_id(&self) -> Option<String> {
        self.meta
            .as_ref()
            .and_then(|meta| meta.sender.custom_attributes.as_ref().and_then(|attrs| attrs.device_id.clone()))
    }
}
