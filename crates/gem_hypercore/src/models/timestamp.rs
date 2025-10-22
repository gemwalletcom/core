use serde::Deserialize;
use serde_serializers::deserialize_u64_from_str_or_int;

#[derive(Debug, Clone, Deserialize)]
pub struct TimestampField {
    #[serde(alias = "time", alias = "nonce", deserialize_with = "deserialize_u64_from_str_or_int")]
    pub value: u64,
}
