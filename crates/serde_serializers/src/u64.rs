use serde::{de, Deserialize, Deserializer};
use serde_json::Value;

pub fn serialize_u64<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_string())
}

pub fn deserialize_u64_from_str<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<u64>().map_err(serde::de::Error::custom)
}

pub fn deserialize_u64_from_str_or_int<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::Number(num) => num.as_u64().ok_or_else(|| de::Error::custom(format!("Invalid number for u64: {}", num))),
        Value::String(s) => {
            if let Some(hex_val) = s.strip_prefix("0x") {
                u64::from_str_radix(hex_val, 16).map_err(|_| de::Error::custom(format!("Invalid hex string for u64: {}", s)))
            } else {
                s.parse::<u64>()
                    .map_err(|_| de::Error::custom(format!("Invalid decimal string for u64: {}", s)))
            }
        }
        _ => Err(de::Error::custom("u64 must be a number or a string")),
    }
}
