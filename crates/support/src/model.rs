use serde::{Deserialize, Serialize};

pub const EVENT_MESSAGE_CREATED: &str = "message_created";
pub const EVENT_CONVERSATION_STATUS_CHANGED: &str = "conversation_status_changed";
pub const EVENT_CONVERSATION_UPDATED: &str = "conversation_updated";

pub const MESSAGE_TYPE_INCOMING: &str = "incoming";
pub const MESSAGE_TYPE_OUTGOING: &str = "outgoing";

pub const MESSAGE_TYPE_INCOMING_INT: i32 = 0;
pub const MESSAGE_TYPE_OUTGOING_INT: i32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatwootWebhookPayload {
    pub event: String,
    pub message_type: Option<String>,
    pub unread_count: Option<i32>,
    pub conversation: Option<Conversation>,
    pub meta: Option<Meta>,
    pub content: Option<String>,
    #[serde(default)]
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub meta: Meta,
    pub unread_count: Option<i32>,
    #[serde(default)]
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: i64,
    pub content: Option<String>,
    pub message_type: i32,
    pub sender: Option<Sender>,
}

impl Message {
    pub fn is_incoming(&self) -> bool {
        self.message_type == MESSAGE_TYPE_INCOMING_INT
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    pub sender: Sender,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomAttributes {
    #[serde(rename = "supportdeviceid", alias = "supportDeviceId", alias = "support_device_id")]
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

    pub fn get_unread(&self) -> Option<i32> {
        self.unread_count
            .or_else(|| self.conversation.as_ref().and_then(|conversation| conversation.unread_count))
    }

    pub fn is_incoming_message(&self) -> bool {
        self.message_type.as_deref() == Some(MESSAGE_TYPE_INCOMING)
    }

    pub fn get_messages(&self) -> &[Message] {
        if !self.messages.is_empty() {
            &self.messages
        } else if let Some(conversation) = &self.conversation {
            &conversation.messages
        } else {
            &[]
        }
    }
}
