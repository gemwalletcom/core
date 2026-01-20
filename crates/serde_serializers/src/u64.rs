use std::fmt;

use serde::{Deserialize, Deserializer, de};

use crate::visitors::{StringOrNumberFromValue, StringOrNumberVisitor};

fn parse_u64_string(value: &str) -> Result<u64, String> {
    if let Some(hex_val) = value.strip_prefix("0x") {
        u64::from_str_radix(hex_val, 16).map_err(|_| format!("Invalid hex string for u64: {value}"))
    } else {
        value.parse::<u64>().map_err(|_| format!("Invalid decimal string for u64: {value}"))
    }
}

fn invalid_number(value: impl fmt::Display) -> String {
    format!("Invalid number for u64: {value}")
}

impl StringOrNumberFromValue for u64 {
    const EXPECTING: &'static str = "a number or string representing u64";

    fn from_str(value: &str) -> Result<Self, String> {
        parse_u64_string(value)
    }

    fn from_u64(value: u64) -> Result<Self, String> {
        Ok(value)
    }

    fn from_i64(value: i64) -> Result<Self, String> {
        if value < 0 {
            return Err(invalid_number(value));
        }
        Ok(value as u64)
    }

    fn from_f64(value: f64) -> Result<Self, String> {
        Err(invalid_number(value))
    }
}

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
    parse_u64_string(&s).map_err(de::Error::custom)
}

pub fn deserialize_u64_from_str_or_int<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(StringOrNumberVisitor::<u64>::new())
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

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestStrOrIntStruct {
        #[serde(deserialize_with = "deserialize_u64_from_str_or_int")]
        pub value: u64,
    }

    #[test]
    fn test_deserialize_u64_from_str_or_int() {
        let result: TestStrOrIntStruct = serde_json::from_str(r#"{"value": 42}"#).unwrap();
        assert_eq!(result.value, 42);

        let result: TestStrOrIntStruct = serde_json::from_str(r#"{"value": "0x2a"}"#).unwrap();
        assert_eq!(result.value, 42);

        let result: TestStrOrIntStruct = serde_json::from_str(r#"{"value": "42"}"#).unwrap();
        assert_eq!(result.value, 42);

        let result: Result<TestStrOrIntStruct, _> = serde_json::from_str(r#"{"value": 1.5}"#);
        assert!(result.is_err());

        let result: Result<TestStrOrIntStruct, _> = serde_json::from_str(r#"{"value": -1}"#);
        assert!(result.is_err());
    }
}
