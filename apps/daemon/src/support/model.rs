use serde::{Deserialize, Serialize};

pub const EVENT_MESSAGE_CREATED: &str = "message_created";
//pub const EVENT_MESSAGE_UPDATED: &str = "message_updated";
//pub const EVENT_CONVERSATION_UPDATED: &str = "conversation_updated";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatwootWebhookPayload {
    pub event: String,
    pub message_type: Option<String>,
    pub conversation: Option<Conversation>,
    pub meta: Option<Meta>,
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
    #[serde(rename = "supportDeviceId")]
    pub support_device_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sender {
    pub custom_attributes: Option<CustomAttributes>,
}

impl ChatwootWebhookPayload {
    pub fn get_support_device_id(&self) -> Option<String> {
        self.conversation
            .as_ref()
            .map(|c| &c.meta)
            .or(self.meta.as_ref())?
            .sender
            .custom_attributes
            .as_ref()?
            .support_device_id
            .clone()
    }
}
