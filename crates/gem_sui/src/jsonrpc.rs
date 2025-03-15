use serde::{Deserialize, Deserializer, Serialize};
use serde_serializers::{deserialize_u64_from_str, serialize_u64};
use std::{fmt::Display, ops::Not};
use sui_types::{
    base_types::{ObjectID, ObjectRef, SequenceNumber},
    digests::ObjectDigest,
};

pub enum SuiRpc {
    GetObject(String, Option<ObjectDataOptions>),
    GetMultipleObjects(Vec<String>, Option<ObjectDataOptions>),
    InspectTransactionBlock(String, String), // sender_address, tx_bytes (base64)
    NormalizedMoveFunction(Vec<String>),
    GetAllCoins { owner: String },
    GetGasPrice,
}

impl Display for SuiRpc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GetObject(_, _) => write!(f, "sui_getObject"),
            Self::GetMultipleObjects(_, _) => write!(f, "sui_multiGetObjects"),
            Self::InspectTransactionBlock(_, _) => write!(f, "sui_devInspectTransactionBlock"),
            Self::NormalizedMoveFunction(_) => write!(f, "sui_getNormalizedMoveFunction"),
            Self::GetAllCoins { owner: _ } => write!(f, "suix_getAllCoins"),
            Self::GetGasPrice => write!(f, "suix_getReferenceGasPrice"),
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
    pub object_id: ObjectID,
    #[serde(deserialize_with = "deserialize_u64_from_str", serialize_with = "serialize_u64")]
    pub version: u64,
    pub digest: ObjectDigest,
    pub owner: Option<Owner>,
    pub content: Option<T>,
}

impl<T> DataObject<T> {
    pub fn to_ref(&self) -> ObjectRef {
        (self.object_id, SequenceNumber::from_u64(self.version), self.digest)
    }
}

impl<T> DataObject<T> {
    pub fn initial_shared_version(&self) -> Option<u64> {
        if let Some(Owner::Shared { initial_shared_version }) = &self.owner {
            Some(*initial_shared_version)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Owner {
    AddressOwner(String),
    Shared { initial_shared_version: u64 },
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
    #[serde(deserialize_with = "deserialize_u64_from_str", serialize_with = "serialize_u64")]
    pub v: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I32 {
    #[serde(deserialize_with = "u32_to_i32", serialize_with = "i32_to_u32")]
    pub bits: i32,
}

fn u32_to_i32<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    let value: u32 = Deserialize::deserialize(deserializer)?;
    Ok(value as i32) // Converts using two's complement
}

fn i32_to_u32<S>(value: &i32, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_u32(value.unsigned_abs())
}
