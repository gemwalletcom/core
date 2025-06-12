use gem_jsonrpc::types::{JsonRpcRequest, JsonRpcRequestConvert};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use serde_serializers::{deserialize_u64_from_str, serialize_u64};
use std::{fmt::Display, ops::Not};
use sui_types::{ObjectDigest, ObjectId};

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

impl JsonRpcRequestConvert for SuiRpc {
    fn to_req(&self, id: u64) -> JsonRpcRequest {
        let val = self;
        let method = val.to_string();

        let params: Vec<Value> = match val {
            SuiRpc::GetObject(object_id, options) => {
                let mut array = vec![Value::String(object_id.into())];
                if let Some(data) = options {
                    let object = serde_json::to_value(data).unwrap();
                    array.push(object);
                }
                array
            }
            SuiRpc::GetMultipleObjects(object_ids, options) => {
                let mut array = vec![Value::Array(object_ids.iter().map(|x| Value::String(x.into())).collect())];
                if let Some(data) = options {
                    let object = serde_json::to_value(data).unwrap();
                    array.push(object);
                }
                array
            }
            SuiRpc::NormalizedMoveFunction(params) => params.iter().map(|x| Value::String(x.into())).collect(),
            SuiRpc::InspectTransactionBlock(sender, tx_bytes) => {
                vec![
                    Value::String(sender.into()),
                    Value::String(tx_bytes.into()),
                    Value::Null, // gas_price
                ]
            }
            SuiRpc::GetAllCoins { owner } => {
                vec![Value::String(owner.into())]
            }
            SuiRpc::GetGasPrice => {
                vec![]
            }
        };

        JsonRpcRequest::new(id, &method, params.into())
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
    pub object_id: ObjectId,
    #[serde(deserialize_with = "deserialize_u64_from_str", serialize_with = "serialize_u64")]
    pub version: u64,
    pub digest: ObjectDigest,
    pub owner: Option<Owner>,
    pub content: Option<T>,
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
