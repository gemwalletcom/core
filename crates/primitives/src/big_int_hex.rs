use std::str::FromStr;

use num_bigint::BigUint;
use num_traits::Num;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone)]
pub struct BigIntHex {
    pub value: BigUint,
}

#[derive(Debug, Clone)]
pub struct BigIntValue {
    pub value: BigUint,
}

impl BigIntHex {
    pub fn as_i64(&self) -> i64 {
        self.value.to_string().parse::<i64>().unwrap_or_default()
    }

    pub fn as_i32(&self) -> i32 {
        self.value.to_string().parse::<i32>().unwrap_or_default()
    }
}

impl Serialize for BigIntHex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.value.to_string())
    }
}

impl<'de> Deserialize<'de> for BigIntHex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let value = BigUint::from_str_radix(&s[2..], 16).map_err(serde::de::Error::custom)?;
        Ok(BigIntHex { value })
    }
}

impl Serialize for BigIntValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.value.to_string())
    }
}

impl<'de> Deserialize<'de> for BigIntValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let value = BigUint::from_str(&s).map_err(serde::de::Error::custom)?;
        Ok(BigIntValue { value })
    }
}
