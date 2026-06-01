use chrono::{DateTime, Utc};
use primitives::{Device, SupportAgent, SupportConversation, SupportConversationStatus, SupportMessage, SupportMessageDeliveryStatus, SupportMessageSender, SupportTypingStatus};
use serde::{Deserialize, Deserializer, Serialize, de::Error as DeError};
use std::collections::HashMap;

pub const EVENT_MESSAGE_CREATED: &str = "message_created";
pub const EVENT_CONVERSATION_STATUS_CHANGED: &str = "conversation_status_changed";
pub const EVENT_CONVERSATION_UPDATED: &str = "conversation_updated";
pub const CHATWOOT_CONTENT_TYPE_TEXT: &str = "text";
pub const CHATWOOT_STATUS_RESOLVED: &str = "resolved";
pub const CHATWOOT_STATUS_OPEN: &str = "open";
pub const CHATWOOT_STATUS_PENDING: &str = "pending";
pub const CHATWOOT_STATUS_SNOOZED: &str = "snoozed";
pub const CHATWOOT_DELIVERY_STATUS_SENT: &str = "sent";
pub const CHATWOOT_DELIVERY_STATUS_DELIVERED: &str = "delivered";
pub const CHATWOOT_DELIVERY_STATUS_READ: &str = "read";

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
    pub id: Option<i64>,
    pub message_type: Option<String>,
    pub private: Option<bool>,
    pub unread_count: Option<i32>,
    pub conversation: Option<Conversation>,
    pub account: Option<Account>,
    pub meta: Option<Meta>,
    pub content: Option<String>,
    pub content_type: Option<String>,
    pub status: Option<String>,
    pub contact_last_seen_at: Option<i64>,
    pub last_activity_at: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_optional_datetime")]
    pub created_at: Option<DateTime<Utc>>,
    pub sender: Option<Sender>,
    #[serde(default)]
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: Option<i64>,
    pub meta: Meta,
    pub status: Option<String>,
    pub unread_count: Option<i32>,
    pub contact_last_seen_at: Option<i64>,
    pub last_activity_at: Option<i64>,
    #[serde(default)]
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: i64,
    pub conversation_id: Option<i64>,
    pub content: Option<String>,
    pub message_type: MessageType,
    pub content_type: Option<String>,
    pub status: Option<String>,
    pub private: Option<bool>,
    pub created_at: i64,
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
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub thumbnail: Option<String>,
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

    pub fn is_public_outgoing_message(&self) -> bool {
        self.is_outgoing_message() && self.private == Some(false)
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

    pub fn support_message(&self) -> Option<SupportMessage> {
        if self.private != Some(false) || self.content_type.as_deref().is_some_and(|content_type| content_type != CHATWOOT_CONTENT_TYPE_TEXT) {
            return None;
        }

        let id = self.id?;
        let conversation_id = self.get_conversation_id()?;
        let content = self.content.clone()?;
        let created_at = self.created_at?;
        let sender = match self.message_type.as_deref()? {
            "incoming" => SupportMessageSender::User,
            "outgoing" => SupportMessageSender::Agent(self.sender.as_ref()?.support_agent()?),
            _ => return None,
        };

        Some(SupportMessage {
            id: id.to_string(),
            conversation_id: conversation_id.to_string(),
            content,
            sender,
            delivery_status: SupportMessageDeliveryStatus::Sent,
            created_at,
        })
    }

    pub fn support_conversation(&self) -> Option<SupportConversation> {
        if let Some(conversation) = &self.conversation {
            return conversation.support_conversation();
        }

        let id = self.id?;
        let messages = support_messages(self.get_messages());
        let first_message = messages.iter().find(|message| message.sender.is_user()).map(|message| message.content.clone());
        let last_message = messages.last().map(|message| message.content.clone());
        let last_activity_at = self
            .last_activity_at
            .and_then(timestamp)
            .or_else(|| messages.last().map(|message| message.created_at))
            .or(self.created_at)?;

        Some(SupportConversation {
            id: id.to_string(),
            status: support_conversation_status(self.status.as_deref()),
            first_message,
            last_message,
            last_activity_at,
            unread_count: self.unread_count.unwrap_or_else(|| unread_count(&messages, self.contact_last_seen_at)),
        })
    }
}

impl Conversation {
    pub fn support_conversation(&self) -> Option<SupportConversation> {
        let id = self.id?;
        let messages = support_messages(&self.messages);
        let first_message = messages.iter().find(|message| message.sender.is_user()).map(|message| message.content.clone());
        let last_message = messages.last().map(|message| message.content.clone());
        let last_activity_at = self.last_activity_at.and_then(timestamp).or_else(|| messages.last().map(|message| message.created_at))?;

        Some(SupportConversation {
            id: id.to_string(),
            status: support_conversation_status(self.status.as_deref()),
            first_message,
            last_message,
            last_activity_at,
            unread_count: self.unread_count.unwrap_or_else(|| unread_count(&messages, self.contact_last_seen_at)),
        })
    }
}

impl Message {
    pub fn support_message(&self) -> Option<SupportMessage> {
        if self.private != Some(false) || self.content_type.as_deref().is_some_and(|content_type| content_type != CHATWOOT_CONTENT_TYPE_TEXT) {
            return None;
        }

        let content = self.content.clone()?;
        let sender = match &self.message_type {
            MessageType::Incoming => SupportMessageSender::User,
            MessageType::Outgoing => SupportMessageSender::Agent(self.sender.as_ref()?.support_agent()?),
        };

        Some(SupportMessage {
            id: self.id.to_string(),
            conversation_id: self.conversation_id?.to_string(),
            content,
            sender,
            delivery_status: support_delivery_status(self.status.as_deref()),
            created_at: timestamp(self.created_at)?,
        })
    }
}

impl Sender {
    pub fn support_agent(&self) -> Option<SupportAgent> {
        let name = self.name.clone()?;
        Some(SupportAgent {
            name,
            avatar_url: self.avatar_url.clone().or_else(|| self.thumbnail.clone()).filter(|value| !value.is_empty()),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatwootSession {
    pub auth_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatwootConfigResponse {
    pub website_channel_config: ChatwootWebsiteChannelConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatwootWebsiteChannelConfig {
    pub auth_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatwootContactResponse {
    pub widget_auth_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatwootMessagesResponse {
    pub payload: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatwootConversationResponse {
    pub id: Option<i64>,
    pub status: Option<String>,
    pub contact_last_seen_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatwootContactUpdate {
    pub identifier: String,
    pub name: String,
    pub custom_attributes: HashMap<String, String>,
}

impl ChatwootContactUpdate {
    pub fn new(device: &Device) -> Self {
        Self {
            identifier: device.id.clone(),
            name: device.model.clone(),
            custom_attributes: HashMap::from([
                ("device_id".to_string(), device.id.clone()),
                ("platform".to_string(), device.platform.as_ref().to_string()),
                ("os".to_string(), device.os.clone()),
                ("device".to_string(), device.model.clone()),
                ("app_version".to_string(), device.version.clone()),
                ("currency".to_string(), device.currency.clone()),
            ]),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatwootMessageInput {
    pub message: ChatwootMessageData,
}

impl ChatwootMessageInput {
    pub fn new(content: String) -> Self {
        Self {
            message: ChatwootMessageData { content },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatwootMessageData {
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatwootTypingInput {
    pub typing_status: String,
}

impl ChatwootTypingInput {
    pub fn new(status: SupportTypingStatus) -> Self {
        let typing_status = match status {
            SupportTypingStatus::On => "on",
            SupportTypingStatus::Off => "off",
        };
        Self {
            typing_status: typing_status.to_string(),
        }
    }
}

pub fn support_messages(messages: &[Message]) -> Vec<SupportMessage> {
    messages.iter().filter_map(Message::support_message).collect()
}

pub fn support_conversation_status(status: Option<&str>) -> SupportConversationStatus {
    match status {
        Some(CHATWOOT_STATUS_RESOLVED) => SupportConversationStatus::Resolved,
        Some(CHATWOOT_STATUS_OPEN) | Some(CHATWOOT_STATUS_PENDING) | Some(CHATWOOT_STATUS_SNOOZED) | None => SupportConversationStatus::Open,
        Some(_) => SupportConversationStatus::Open,
    }
}

pub fn support_delivery_status(status: Option<&str>) -> SupportMessageDeliveryStatus {
    match status {
        Some(CHATWOOT_DELIVERY_STATUS_SENT) | Some(CHATWOOT_DELIVERY_STATUS_DELIVERED) | Some(CHATWOOT_DELIVERY_STATUS_READ) | None => SupportMessageDeliveryStatus::Sent,
        Some(_) => SupportMessageDeliveryStatus::Failed,
    }
}

pub fn unread_count(messages: &[SupportMessage], contact_last_seen_at: Option<i64>) -> i32 {
    let Some(contact_last_seen_at) = contact_last_seen_at else {
        return 0;
    };
    messages
        .iter()
        .filter(|message| message.sender.is_agent() && message.created_at.timestamp() > contact_last_seen_at)
        .count() as i32
}

pub fn timestamp(value: i64) -> Option<DateTime<Utc>> {
    DateTime::<Utc>::from_timestamp(value, 0)
}

fn deserialize_optional_datetime<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    match value {
        Some(serde_json::Value::String(value)) => Ok(Some(value.parse::<DateTime<Utc>>().map_err(D::Error::custom)?)),
        Some(serde_json::Value::Number(value)) => Ok(value.as_i64().and_then(timestamp)),
        Some(_) | None => Ok(None),
    }
}
