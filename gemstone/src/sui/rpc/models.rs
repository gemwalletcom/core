use num_bigint::BigInt;
use serde::Deserialize;
use serde_serializers::*;

use sui_transaction_builder::unresolved::Input;
use sui_types::{Address, Digest};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoinAsset {
    pub coin_object_id: Address,
    pub coin_type: String,
    pub digest: Digest,
    #[serde(deserialize_with = "deserialize_bigint_from_str", serialize_with = "serialize_bigint")]
    pub balance: BigInt,
    #[serde(deserialize_with = "deserialize_u64_from_str", serialize_with = "serialize_u64")]
    pub version: u64,
}

impl CoinAsset {
    pub fn to_input(&self) -> Input {
        Input::owned(self.coin_object_id, self.version, self.digest)
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
pub struct InspectResult {
    pub effects: InspectEffects,
    pub events: serde_json::Value,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectEvent<T> {
    pub package_id: String,
    pub transaction_module: String,
    pub parsed_json: T,
    pub r#type: String,
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
