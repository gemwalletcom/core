use serde::{Deserialize, Serialize};

pub const EVENT_MESSAGE_CREATED: &str = "message_created";
pub const MESSAGE_TYPE_OUTGOING: &str = "outgoing";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatwootWebhookPayload {
    pub event: String,
    pub message_type: String,
    pub conversation: Conversation,
    pub content: Option<String>,
    pub sender: Option<Sender>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: i64,
    pub meta: ConversationMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMeta {
    pub sender: MetaSender,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaSender {
    pub custom_attributes: Option<CustomAttributes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomAttributes {
    #[serde(rename = "deviceId")]
    pub device_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sender {
    pub name: String,
}

impl ChatwootWebhookPayload {
    pub fn get_device_id(&self) -> Option<String> {
        self.conversation
            .meta
            .sender
            .custom_attributes
            .as_ref()
            .and_then(|attrs| attrs.device_id.clone())
    }
}
