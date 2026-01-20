use num_bigint::BigUint;
use serde::{Deserialize, de};

use primitives::hex::decode_hex;

pub fn serialize_biguint<S>(value: &BigUint, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_string())
}

pub fn serialize_biguint_to_hex_str<S>(value: &BigUint, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&format!("0x{}", value.to_str_radix(16)))
}

pub fn deserialize_biguint_from_str<'de, D>(deserializer: D) -> Result<BigUint, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: String = de::Deserialize::deserialize(deserializer)?;
    s.parse::<BigUint>().map_err(de::Error::custom)
}

pub fn deserialize_option_biguint_from_str<'de, D>(deserializer: D) -> Result<Option<BigUint>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(value) => value.parse::<BigUint>().map(Some).map_err(de::Error::custom),
        None => Ok(None),
    }
}

pub fn deserialize_biguint_from_hex_str<'de, D>(deserializer: D) -> Result<BigUint, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: String = de::Deserialize::deserialize(deserializer)?;
    biguint_from_prefixed_hex(&s)
}

pub fn deserialize_biguint_from_option_hex_str<'de, D>(deserializer: D) -> Result<Option<BigUint>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        Some(s) => biguint_from_prefixed_hex(&s).map(Some),
        None => Ok(None),
    }
}

fn biguint_from_prefixed_hex<E>(hex_value: &str) -> Result<BigUint, E>
where
    E: de::Error,
{
    let trimmed = hex_value.trim();
    if !trimmed.starts_with("0x") {
        return Err(de::Error::custom(format!("Invalid hex string: {hex_value}")));
    }
    biguint_from_hex_str(trimmed).map_err(de::Error::custom)
}

pub fn biguint_from_hex_str(hex_value: &str) -> Result<BigUint, Box<dyn std::error::Error + Send + Sync>> {
    let bytes = decode_hex(hex_value)?;
    Ok(BigUint::from_bytes_be(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[test]
    fn test_biguint_from_hex_str() {
        assert_eq!(biguint_from_hex_str("0x1a").unwrap(), BigUint::from(26u32));
        assert_eq!(biguint_from_hex_str("1a").unwrap(), BigUint::from(26u32));
        assert_eq!(biguint_from_hex_str("0x0").unwrap(), BigUint::from(0u32));
        assert_eq!(biguint_from_hex_str("ff").unwrap(), BigUint::from(255u32));
        assert!(biguint_from_hex_str("xyz").is_err());
    }

    #[derive(Deserialize)]
    struct HexStruct {
        #[serde(deserialize_with = "deserialize_biguint_from_hex_str")]
        value: BigUint,
    }

    #[test]
    fn test_deserialize_biguint_from_hex_str() {
        let deserialized: HexStruct = serde_json::from_str(r#"{"value":"0x1a"}"#).unwrap();
        assert_eq!(deserialized.value, BigUint::from(26u32));

        let result: Result<HexStruct, _> = serde_json::from_str(r#"{"value":"1a"}"#);
        assert!(result.is_err());
    }

    #[derive(Deserialize)]
    struct OptionHexStruct {
        #[serde(default, deserialize_with = "deserialize_biguint_from_option_hex_str")]
        value: Option<BigUint>,
    }

    #[test]
    fn test_deserialize_biguint_from_option_hex_str() {
        let deserialized: OptionHexStruct = serde_json::from_str(r#"{"value":"0x0"}"#).unwrap();
        assert_eq!(deserialized.value, Some(BigUint::from(0u32)));

        let deserialized: OptionHexStruct = serde_json::from_str(r#"{}"#).unwrap();
        assert_eq!(deserialized.value, None);
    }
}
