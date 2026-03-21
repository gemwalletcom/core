use super::Coin;
use super::long::deserialize_u64_from_long_or_int;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IbcTransferValue {
    pub source_port: String,
    pub source_channel: String,
    pub token: Coin,
    pub sender: String,
    pub receiver: String,
    #[serde(deserialize_with = "deserialize_u64_from_long_or_int")]
    pub timeout_timestamp: u64,
    pub memo: String,
}
