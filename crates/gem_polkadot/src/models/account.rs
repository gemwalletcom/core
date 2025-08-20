use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_u64_from_str, deserialize_bigint_from_str, serialize_bigint};
use num_bigint::BigInt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolkadotAccountBalance {
    #[serde(serialize_with = "serialize_bigint", deserialize_with = "deserialize_bigint_from_str")]
    pub free: BigInt,
    #[serde(serialize_with = "serialize_bigint", deserialize_with = "deserialize_bigint_from_str")]
    pub reserved: BigInt,
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub nonce: u64,
}
