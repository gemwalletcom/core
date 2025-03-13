use super::serializer::*;
use num_bigint::BigInt;
use serde::Deserialize;

use sui_types::{
    base_types::{ObjectID, ObjectRef},
    digests::ObjectDigest,
};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoinAsset {
    pub coin_object_id: ObjectID,
    pub coin_type: String,
    pub digest: ObjectDigest,
    #[serde(deserialize_with = "deserialize_bigint", serialize_with = "serialize_bigint")]
    pub balance: BigInt,
    #[serde(deserialize_with = "deserialize_u64_from_str", serialize_with = "serialize_u64")]
    pub version: u64,
}

impl CoinAsset {
    pub fn to_ref(&self) -> ObjectRef {
        (self.coin_object_id, self.version.into(), self.digest)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoinResponse {
    pub data: Vec<CoinAsset>,
    pub next_cursor: Option<String>,
    pub has_next_page: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectResults {
    pub effects: InspectEffects,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectEffects {
    pub gas_used: InspectGasUsed,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectGasUsed {
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub computation_cost: u64,
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub storage_cost: u64,
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub storage_rebate: u64,
}
