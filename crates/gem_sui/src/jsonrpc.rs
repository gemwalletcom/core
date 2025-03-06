use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::{fmt::Display, ops::Not};

pub enum SuiRpc {
    GetObject(String, Option<ObjectDataOptions>),
    GetMultipleObjects(Vec<String>, Option<ObjectDataOptions>),
    InspectTransactionBlock(String, String), // sender_address, tx_bytes (base64)
    NormalizedMoveFunction(Vec<String>),
}

impl Display for SuiRpc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GetObject(_, _) => write!(f, "sui_getObject"),
            Self::GetMultipleObjects(_, _) => write!(f, "sui_multiGetObjects"),
            Self::InspectTransactionBlock(_, _) => write!(f, "sui_devInspectTransactionBlock"),
            Self::NormalizedMoveFunction(_) => write!(f, "sui_getNormalizedMoveFunction"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectDataOptions {
    #[serde(skip_serializing_if = "Not::not")]
    pub show_type: bool,
    #[serde(skip_serializing_if = "Not::not")]
    pub show_owner: bool,
    #[serde(skip_serializing_if = "Not::not")]
    pub show_display: bool,
    #[serde(skip_serializing_if = "Not::not")]
    pub show_content: bool,
    #[serde(skip_serializing_if = "Not::not")]
    pub show_bcs: bool,
}

impl Default for ObjectDataOptions {
    fn default() -> Self {
        Self {
            show_type: false,
            show_owner: true,
            show_display: false,
            show_content: true,
            show_bcs: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiData<T> {
    pub data: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataObject<T> {
    pub object_id: String,
    pub version: String,
    pub digest: String,
    pub owner: Option<Value>,
    pub content: Option<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveObject<T> {
    pub r#type: String,
    pub fields: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveObjectId {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionU64 {
    pub is_none: bool,
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub v: u64,
}

fn deserialize_u64_from_str<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = serde::Deserialize::deserialize(deserializer)?;
    s.parse::<u64>().map_err(serde::de::Error::custom)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I32 {
    #[serde(deserialize_with = "u32_to_i32")]
    pub bits: i32,
}

fn u32_to_i32<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    let value: u32 = Deserialize::deserialize(deserializer)?;
    Ok(value as i32) // Converts using two’s complement
}
