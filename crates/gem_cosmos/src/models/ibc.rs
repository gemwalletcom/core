use super::Coin;
use serde::Deserialize;
use serde_serializers::deserialize_u64_from_str_or_int;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IbcTransferValue {
    pub source_port: String,
    pub source_channel: String,
    pub token: Coin,
    pub sender: String,
    pub receiver: String,
    #[serde(default, deserialize_with = "deserialize_u64_from_str_or_int")]
    pub timeout_timestamp: u64,
    #[serde(default)]
    pub memo: String,
}
