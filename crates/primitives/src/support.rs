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
pub enum SupportMessageDirection {
    Incoming,
    Outgoing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
#[serde(rename_all = "lowercase")]
pub enum SupportMessageDeliveryStatus {
    Sending,
    Sent,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable, Equatable")]
#[serde(rename_all = "camelCase")]
pub struct SupportAgent {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable, Equatable")]
#[serde(rename_all = "camelCase")]
pub struct SupportMessage {
    pub id: String,
    pub conversation_id: String,
    pub content: String,
    pub direction: SupportMessageDirection,
    pub delivery_status: SupportMessageDeliveryStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<SupportAgent>,
    pub created_at: DateTime<Utc>,
}
