use serde::{Deserialize, Serialize};

pub const EVENT_MESSAGE_CREATED: &str = "message_created";
pub const EVENT_CONVERSATION_STATUS_CHANGED: &str = "conversation_status_changed";
pub const EVENT_CONVERSATION_UPDATED: &str = "conversation_updated";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(from = "i32", into = "i32")]
pub enum MessageType {
    Incoming,
    Outgoing,
}

impl From<i32> for MessageType {
    fn from(value: i32) -> Self {
        match value {
            1 => MessageType::Outgoing,
            _ => MessageType::Incoming,
        }
    }
}

impl From<MessageType> for i32 {
    fn from(value: MessageType) -> Self {
        match value {
            MessageType::Incoming => 0,
            MessageType::Outgoing => 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatwootWebhookPayload {
    pub event: String,
    pub message_type: Option<String>,
    pub unread_count: Option<i32>,
    pub conversation: Option<Conversation>,
    pub account: Option<Account>,
    pub meta: Option<Meta>,
    pub content: Option<String>,
    #[serde(default)]
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: Option<i64>,
    pub meta: Meta,
    pub unread_count: Option<i32>,
    #[serde(default)]
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: i64,
    pub content: Option<String>,
    pub message_type: MessageType,
    pub sender: Option<Sender>,
}

impl Message {
    pub fn is_incoming(&self) -> bool {
        self.message_type == MessageType::Incoming
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    pub sender: Sender,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomAttributes {
    pub device_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sender {
    pub custom_attributes: Option<CustomAttributes>,
}

impl ChatwootWebhookPayload {
    pub fn get_device_id(&self) -> Option<String> {
        let attrs = self.conversation.as_ref().map(|c| &c.meta).or(self.meta.as_ref())?.sender.custom_attributes.as_ref()?;
        attrs.device_id.clone()
    }

    pub fn get_unread(&self) -> Option<i32> {
        self.unread_count.or_else(|| self.conversation.as_ref().and_then(|conversation| conversation.unread_count))
    }

    pub fn is_outgoing_message(&self) -> bool {
        self.message_type.as_deref() == Some("outgoing")
    }

    pub fn is_incoming_message(&self) -> bool {
        self.message_type.as_deref() == Some("incoming")
    }

    pub fn get_account_id(&self) -> Option<i64> {
        self.account.as_ref().map(|a| a.id)
    }

    pub fn get_conversation_id(&self) -> Option<i64> {
        self.conversation.as_ref().and_then(|c| c.id)
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
