use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "lowercase")]
pub enum SupportConversationStatus {
    Open,
    Resolved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "lowercase")]
pub enum SupportMessageDeliveryStatus {
    Sending,
    Sent,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[typeshare(swift = "Sendable, Equatable")]
#[serde(rename_all = "camelCase")]
pub struct SupportAgent {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(tag = "type", content = "data", rename_all = "lowercase")]
pub enum SupportMessageSender {
    User,
    Agent(SupportAgent),
}

impl SupportMessageSender {
    pub fn is_user(&self) -> bool {
        match self {
            Self::User => true,
            Self::Agent(_) => false,
        }
    }

    pub fn is_agent(&self) -> bool {
        match self {
            Self::User => false,
            Self::Agent(_) => true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[typeshare(swift = "Sendable, Equatable, Hashable, Identifiable")]
#[serde(rename_all = "camelCase")]
pub struct SupportConversation {
    pub id: String,
    pub status: SupportConversationStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_message: Option<String>,
    pub last_activity_at: DateTime<Utc>,
    pub unread_count: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[typeshare(swift = "Sendable, Equatable")]
#[serde(rename_all = "camelCase")]
pub struct SupportMessage {
    pub id: String,
    pub conversation_id: String,
    pub content: String,
    pub sender: SupportMessageSender,
    pub delivery_status: SupportMessageDeliveryStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[typeshare(swift = "Sendable, Equatable")]
#[serde(rename_all = "camelCase")]
pub struct SupportMessageInput {
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "lowercase")]
pub enum SupportTypingStatus {
    On,
    Off,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[typeshare(swift = "Sendable, Equatable")]
#[serde(rename_all = "camelCase")]
pub struct SupportTyping {
    pub status: SupportTypingStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
#[typeshare(swift = "Sendable")]
pub enum SupportStreamEvent {
    Message(SupportMessage),
    Conversation(SupportConversation),
}
