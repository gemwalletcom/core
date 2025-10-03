use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_bigint_from_str, deserialize_u64_from_str, serialize_bigint, serialize_u64};
use sui_transaction_builder::unresolved::Input;
use sui_types::{Address, Digest};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoinResponse {
    pub data: Vec<CoinAsset>,
    pub next_cursor: Option<String>,
    pub has_next_page: bool,
}
