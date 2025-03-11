use gem_sui::jsonrpc::deserialize_u64_from_str;
use serde::Deserialize;
use std::str::FromStr;
use sui_types::{
    base_types::{ObjectID, ObjectRef, SequenceNumber},
    digests::ObjectDigest,
};

#[derive(Debug, Clone, Deserialize)]
pub struct CoinAsset {
    pub coin_object_id: ObjectID,
    pub coin_type: String,
    pub digest: String,
    pub balance: u64,
    pub version: SequenceNumber,
}

impl From<CoinData> for CoinAsset {
    fn from(coin_data: CoinData) -> Self {
        Self {
            coin_type: coin_data.coin_type,
            digest: coin_data.digest,
            balance: coin_data.balance.parse().unwrap_or(0),
            coin_object_id: ObjectID::from_hex_literal(&coin_data.coin_object_id).unwrap(),
            version: SequenceNumber::from_u64(coin_data.version.parse().unwrap_or(0)),
        }
    }
}

impl CoinAsset {
    pub fn to_ref(&self) -> ObjectRef {
        (self.coin_object_id, self.version, ObjectDigest::from_str(&self.digest).unwrap())
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoinResponse {
    pub data: Vec<CoinData>,
    pub next_cursor: Option<String>,
    pub has_next_page: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoinData {
    pub coin_object_id: String,
    pub coin_type: String,
    pub balance: String,
    pub digest: String,
    pub version: String,
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
