use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_u64_from_str;

use super::coin::CoinData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub sequence_number: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource<T> {
    #[serde(rename = "type")]
    pub type_field: String,
    pub data: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceData {
    pub coin: Option<CoinData>,
}