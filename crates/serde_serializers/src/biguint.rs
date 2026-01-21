use num_bigint::BigUint;
use serde::{Deserialize, de};

fn parse_biguint_hex(value: &str) -> Result<BigUint, String> {
    let hex_value = value.strip_prefix("0x").unwrap_or(value);
    if hex_value.is_empty() {
        return Ok(BigUint::from(0u32));
    }
    BigUint::parse_bytes(hex_value.as_bytes(), 16).ok_or_else(|| format!("Invalid hex string: {value}"))
}

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
    parse_biguint_hex(&s).map_err(de::Error::custom)
}

pub fn deserialize_biguint_from_option_hex_str<'de, D>(deserializer: D) -> Result<Option<BigUint>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        Some(s) => parse_biguint_hex(&s).map(Some).map_err(de::Error::custom),
        None => Ok(None),
    }
}

pub fn biguint_from_hex_str(hex_value: &str) -> Result<BigUint, Box<dyn std::error::Error + Send + Sync>> {
    parse_biguint_hex(hex_value).map_err(|err| err.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct HexStruct {
        #[serde(deserialize_with = "deserialize_biguint_from_hex_str")]
        value: BigUint,
    }

    #[derive(Deserialize)]
    struct OptionHexStruct {
        #[serde(default, deserialize_with = "deserialize_biguint_from_option_hex_str")]
        value: Option<BigUint>,
    }

    #[test]
    fn test_biguint_hex_deserialization() {
        assert_eq!(biguint_from_hex_str("0x1a").unwrap(), BigUint::from(26u32));
        assert_eq!(biguint_from_hex_str("1a").unwrap(), BigUint::from(26u32));
        assert_eq!(biguint_from_hex_str("0x0").unwrap(), BigUint::from(0u32));
        assert_eq!(biguint_from_hex_str("0x").unwrap(), BigUint::from(0u32));
        assert_eq!(biguint_from_hex_str("ff").unwrap(), BigUint::from(255u32));
        assert!(biguint_from_hex_str("xyz").is_err());

        let hex_cases = [(r#"{"value":"0x1a"}"#, BigUint::from(26u32)), (r#"{"value":"1a"}"#, BigUint::from(26u32))];
        for (json, expected) in hex_cases {
            let deserialized: HexStruct = serde_json::from_str(json).unwrap();
            assert_eq!(deserialized.value, expected);
        }

        let deserialized: OptionHexStruct = serde_json::from_str(r#"{"value":"0x0"}"#).unwrap();
        assert_eq!(deserialized.value, Some(BigUint::from(0u32)));

        let deserialized: OptionHexStruct = serde_json::from_str(r#"{}"#).unwrap();
        assert_eq!(deserialized.value, None);
    }
}
