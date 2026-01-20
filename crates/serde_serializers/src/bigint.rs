use num_bigint::BigInt;
use serde::{Deserialize, de};

fn parse_bigint_hex(value: &str) -> Result<BigInt, String> {
    let hex_value = value.strip_prefix("0x").unwrap_or(value);
    if hex_value.is_empty() {
        return Ok(BigInt::from(0));
    }
    BigInt::parse_bytes(hex_value.as_bytes(), 16).ok_or_else(|| format!("Invalid hex string: {value}"))
}

fn parse_bigint_str(value: &str) -> Result<BigInt, String> {
    if value.starts_with("0x") {
        parse_bigint_hex(value)
    } else {
        value.parse::<BigInt>().map_err(|err| err.to_string())
    }
}

pub fn serialize_bigint<S>(value: &BigInt, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_string())
}

pub fn deserialize_bigint_from_str<'de, D>(deserializer: D) -> Result<BigInt, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: String = de::Deserialize::deserialize(deserializer)?;
    parse_bigint_str(&s).map_err(de::Error::custom)
}

pub fn deserialize_option_bigint_from_str<'de, D>(deserializer: D) -> Result<Option<BigInt>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(str_val) => parse_bigint_str(&str_val).map(Some).map_err(de::Error::custom),
        None => Ok(None),
    }
}

pub fn bigint_from_hex_str(hex_str: &str) -> Result<BigInt, Box<dyn std::error::Error + Send + Sync>> {
    parse_bigint_hex(hex_str).map_err(|err| err.into())
}

pub fn deserialize_bigint_vec_from_hex_str<'de, D>(deserializer: D) -> Result<Vec<BigInt>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let hex_strings: Vec<String> = de::Deserialize::deserialize(deserializer)?;
    hex_strings.into_iter().map(|hex_str| bigint_from_hex_str(&hex_str).map_err(de::Error::custom)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigInt;
    use serde::{Deserialize, Serialize};
    use serde_json;

    #[derive(Serialize, Deserialize)]
    struct TestStruct {
        #[serde(serialize_with = "serialize_bigint", deserialize_with = "deserialize_bigint_from_str")]
        value: BigInt,
    }

    #[test]
    fn test_serialize_bigint() {
        let value = BigInt::parse_bytes(b"12345678901234567890", 10).unwrap();
        let test_struct = TestStruct { value };
        let serialized = serde_json::to_string(&test_struct).unwrap();
        assert_eq!(serialized, r#"{"value":"12345678901234567890"}"#);
    }

    #[test]
    fn test_deserialize_bigint() {
        let json_data = r#"{"value":"12345678901234567890"}"#;
        let deserialized: TestStruct = serde_json::from_str(json_data).unwrap();
        let expected_value = BigInt::parse_bytes(b"12345678901234567890", 10).unwrap();
        assert_eq!(deserialized.value, expected_value);
    }

    #[test]
    fn test_deserialize_bigint_hex() {
        let json_data = r#"{"value":"0xff"}"#;
        let deserialized: TestStruct = serde_json::from_str(json_data).unwrap();
        let expected_value = BigInt::from(255);
        assert_eq!(deserialized.value, expected_value);
    }

    #[test]
    fn test_deserialize_bigint_hex_zero() {
        let json_data = r#"{"value":"0x0"}"#;
        let deserialized: TestStruct = serde_json::from_str(json_data).unwrap();
        let expected_value = BigInt::from(0);
        assert_eq!(deserialized.value, expected_value);
    }

    #[test]
    fn test_deserialize_bigint_hex_empty() {
        let json_data = r#"{"value":"0x"}"#;
        let deserialized: TestStruct = serde_json::from_str(json_data).unwrap();
        let expected_value = BigInt::from(0);
        assert_eq!(deserialized.value, expected_value);
    }

    #[test]
    fn test_bigint_from_hex_str() {
        assert_eq!(bigint_from_hex_str("0x1a").unwrap(), BigInt::from(26));
        assert_eq!(bigint_from_hex_str("1a").unwrap(), BigInt::from(26));
        assert_eq!(bigint_from_hex_str("0x0").unwrap(), BigInt::from(0));
        assert_eq!(bigint_from_hex_str("0x").unwrap(), BigInt::from(0));
        assert_eq!(bigint_from_hex_str("ff").unwrap(), BigInt::from(255));
        assert!(bigint_from_hex_str("xyz").is_err());
    }

    #[derive(Deserialize)]
    struct TestVecStruct {
        #[serde(deserialize_with = "deserialize_bigint_vec_from_hex_str")]
        values: Vec<BigInt>,
    }

    #[test]
    fn test_deserialize_bigint_vec_from_hex_str() {
        let json_data = r#"{"values":["0x1a","0xff","0x0"]}"#;
        let deserialized: TestVecStruct = serde_json::from_str(json_data).unwrap();

        assert_eq!(deserialized.values.len(), 3);
        assert_eq!(deserialized.values[0], BigInt::from(26));
        assert_eq!(deserialized.values[1], BigInt::from(255));
        assert_eq!(deserialized.values[2], BigInt::from(0));
    }
}
