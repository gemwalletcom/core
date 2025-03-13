use num_bigint::BigInt;
use serde::{Deserialize, Deserializer, Serializer};
use std::str::FromStr;

/// Deserialize a string into a BigInt
pub fn deserialize_bigint<'de, D>(deserializer: D) -> Result<BigInt, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    BigInt::from_str(&s).map_err(serde::de::Error::custom)
}

/// Serialize a BigInt as a string
pub fn serialize_bigint<S>(value: &BigInt, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.to_string())
}