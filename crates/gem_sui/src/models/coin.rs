use serde::{Deserialize, Serialize};

#[cfg(feature = "rpc")]
use num_bigint::BigInt;
#[cfg(feature = "rpc")]
use serde_serializers::deserialize_bigint_from_str;

#[cfg(feature = "rpc")]
use super::account::Owner;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiCoin {
    pub coin_type: String,
    pub coin_object_id: String,
    pub balance: String,
    pub version: String,
    pub digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiCoinBalance {
    pub coin_type: String,
    pub total_balance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiCoinMetadata {
    pub decimals: i32,
    pub name: String,
    pub symbol: String,
}

#[cfg(feature = "rpc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    pub coin_type: String,
    #[serde(deserialize_with = "deserialize_bigint_from_str")]
    pub total_balance: BigInt,
}

#[cfg(feature = "rpc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceChange {
    pub owner: Owner,
    #[serde(rename = "coinType")]
    pub coin_type: String,
    #[serde(deserialize_with = "deserialize_bigint_from_str")]
    pub amount: BigInt,
}
