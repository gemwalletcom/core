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
    if let Some(hex_val) = s.strip_prefix("0x") {
        u64::from_str_radix(hex_val, 16).map_err(|_| de::Error::custom(format!("Invalid hex string for u64: {s}")))
    } else {
        s.parse::<u64>().map_err(serde::de::Error::custom)
    }
}

pub fn deserialize_u64_from_str_or_int<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::Number(num) => num.as_u64().ok_or_else(|| de::Error::custom(format!("Invalid number for u64: {num}"))),
        Value::String(s) => {
            if let Some(hex_val) = s.strip_prefix("0x") {
                u64::from_str_radix(hex_val, 16).map_err(|_| de::Error::custom(format!("Invalid hex string for u64: {s}")))
            } else {
                s.parse::<u64>().map_err(|_| de::Error::custom(format!("Invalid decimal string for u64: {s}")))
            }
        }
        _ => Err(de::Error::custom("u64 must be a number or a string")),
    }
}


pub fn deserialize_option_u64_from_str<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(str_val) => str_val.parse::<u64>().map(Some).map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestStruct {
        #[serde(default, deserialize_with = "deserialize_option_u64_from_str")]
        pub gas_used: Option<u64>,
    }

    #[test]
    fn test_deserialize_option_u64_from_str() {
        // Test with string value
        let json = r#"{"gas_used": "123"}"#;
        let result: TestStruct = serde_json::from_str(json).unwrap();
        assert_eq!(result.gas_used, Some(123));

        // Test with null value
        let json = r#"{"gas_used": null}"#;
        let result: TestStruct = serde_json::from_str(json).unwrap();
        assert_eq!(result.gas_used, None);

        // Test with missing field (should use default)
        let json = r#"{}"#;
        let result: TestStruct = serde_json::from_str(json).unwrap();
        assert_eq!(result.gas_used, None);
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestMixedStruct {
        #[serde(deserialize_with = "deserialize_u64_from_str")]
        pub value: u64,
    }

    #[test]
    fn test_deserialize_u64_from_str_with_hex() {
        // Test with 0x prefixed hex
        let json = r#"{"value": "0x1a2b"}"#;
        let result: TestMixedStruct = serde_json::from_str(json).unwrap();
        assert_eq!(result.value, 6699); // 0x1a2b = 6699

        // Test with zero hex
        let json = r#"{"value": "0x0"}"#;
        let result: TestMixedStruct = serde_json::from_str(json).unwrap();
        assert_eq!(result.value, 0);

        // Test with decimal string
        let json = r#"{"value": "12345"}"#;
        let result: TestMixedStruct = serde_json::from_str(json).unwrap();
        assert_eq!(result.value, 12345);

        // Test with zero decimal
        let json = r#"{"value": "0"}"#;
        let result: TestMixedStruct = serde_json::from_str(json).unwrap();
        assert_eq!(result.value, 0);
    }
}
