use crate::AssetId;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};
use typeshare::typeshare;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, AsRefStr, EnumString)]
#[strum(serialize_all = "camelCase")]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable, Equatable")]
pub enum CoreEmoji {
    Gift,
    Gem,
    Party,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[typeshare(swift = "Sendable, Equatable")]
#[serde(rename_all = "camelCase", tag = "type", content = "value")]
pub enum CoreListItemIcon {
    Emoji(CoreEmoji),
    Asset(AssetId),
    Image(String),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, AsRefStr, EnumString)]
#[strum(serialize_all = "camelCase")]
#[serde(rename_all = "camelCase")]
#[typeshare(swift = "Sendable, Equatable")]
pub enum CoreListItemBadge {
    New,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable, Equatable")]
#[serde(rename_all = "camelCase")]
pub struct CoreListItem {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subvalue: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<CoreListItemIcon>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge: Option<CoreListItemBadge>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}
