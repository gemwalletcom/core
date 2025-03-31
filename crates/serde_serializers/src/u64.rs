use serde::{Deserialize, Deserializer};

pub fn serialize_u64<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_string())
}

pub fn serialize_optional_u64<S>(value: &Option<u64>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match value {
        Some(value) => serializer.serialize_str(&value.to_string()),
        None => serializer.serialize_none(),
    }
}

pub fn deserialize_u64_from_str<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<u64>().map_err(serde::de::Error::custom)
}

pub fn deserialize_optional_u64_from_str<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    let result: Result<String, D::Error> = Deserialize::deserialize(deserializer);
    match result {
        Ok(s) => {
            if s.is_empty() {
                Ok(Some(0))
            } else {
                s.parse::<u64>().map_err(serde::de::Error::custom).map(Some)
            }
        }
        Err(_) => Ok(None),
    }
}
