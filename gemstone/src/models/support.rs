use crate::models::custom_types::DateTimeUtc;
use primitives::{
    SupportAgent, SupportConversation, SupportConversationStatus, SupportMessage, SupportMessageDeliveryStatus, SupportMessageInput, SupportMessageSender, SupportTyping,
    SupportTypingStatus,
};

pub type GemSupportAgent = SupportAgent;
pub type GemSupportConversation = SupportConversation;
pub type GemSupportConversationStatus = SupportConversationStatus;
pub type GemSupportMessage = SupportMessage;
pub type GemSupportMessageDeliveryStatus = SupportMessageDeliveryStatus;
pub type GemSupportMessageInput = SupportMessageInput;
pub type GemSupportMessageSender = SupportMessageSender;
pub type GemSupportTyping = SupportTyping;
pub type GemSupportTypingStatus = SupportTypingStatus;

#[uniffi::remote(Enum)]
pub enum GemSupportConversationStatus {
    Open,
    Resolved,
}

#[uniffi::remote(Enum)]
pub enum GemSupportMessageDeliveryStatus {
    Sending,
    Sent,
    Failed,
}

#[uniffi::remote(Record)]
pub struct GemSupportAgent {
    pub name: String,
    pub avatar_url: Option<String>,
}

#[uniffi::remote(Enum)]
pub enum GemSupportMessageSender {
    User,
    Agent(GemSupportAgent),
}

#[uniffi::remote(Record)]
pub struct GemSupportConversation {
    pub id: String,
    pub status: GemSupportConversationStatus,
    pub first_message: Option<String>,
    pub last_message: Option<String>,
    pub last_activity_at: DateTimeUtc,
    pub unread_count: i32,
}

#[uniffi::remote(Record)]
pub struct GemSupportMessage {
    pub id: String,
    pub conversation_id: String,
    pub content: String,
    pub sender: GemSupportMessageSender,
    pub delivery_status: GemSupportMessageDeliveryStatus,
    pub created_at: DateTimeUtc,
}

#[uniffi::remote(Record)]
pub struct GemSupportMessageInput {
    pub content: String,
}

#[uniffi::remote(Enum)]
pub enum GemSupportTypingStatus {
    On,
    Off,
}

#[uniffi::remote(Record)]
pub struct GemSupportTyping {
    pub status: GemSupportTypingStatus,
}
