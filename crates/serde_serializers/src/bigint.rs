use num_bigint::BigInt;
use serde::de;

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
    
    // Handle hex strings (0x prefixed)
    if let Some(hex_str) = s.strip_prefix("0x") {
        if hex_str.is_empty() {
            return Ok(BigInt::from(0));
        }
        BigInt::parse_bytes(hex_str.as_bytes(), 16).ok_or_else(|| de::Error::custom(format!("Invalid hex string: {s}")))
    } else {
        // Handle regular decimal strings
        s.parse::<BigInt>().map_err(de::Error::custom)
    }
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
}
