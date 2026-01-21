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
    const EXPECTING: &'static str = "a string or integer representing u64";

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
        Some(str_val) => parse_u64_string(&str_val).map(Some).map_err(de::Error::custom),
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

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestMixedStruct {
        #[serde(deserialize_with = "deserialize_u64_from_str")]
        pub value: u64,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestStrOrIntStruct {
        #[serde(deserialize_with = "deserialize_u64_from_str_or_int")]
        pub value: u64,
    }

    #[test]
    fn test_u64_deserialization() {
        let option_cases = [
            (r#"{"gas_used": "123"}"#, Some(123u64)),
            (r#"{"gas_used": "0x2a"}"#, Some(42)),
            (r#"{"gas_used": null}"#, None),
            (r#"{}"#, None),
        ];
        for (json, expected) in option_cases {
            let result: TestStruct = serde_json::from_str(json).unwrap();
            assert_eq!(result.gas_used, expected);
        }

        let str_cases = [
            (r#"{"value": "0x1a2b"}"#, 6699u64),
            (r#"{"value": "0x0"}"#, 0),
            (r#"{"value": "12345"}"#, 12345),
            (r#"{"value": "0"}"#, 0),
        ];
        for (json, expected) in str_cases {
            let result: TestMixedStruct = serde_json::from_str(json).unwrap();
            assert_eq!(result.value, expected);
        }

        let mixed_cases = [(r#"{"value": 42}"#, 42u64), (r#"{"value": "0x2a"}"#, 42), (r#"{"value": "42"}"#, 42)];
        for (json, expected) in mixed_cases {
            let result: TestStrOrIntStruct = serde_json::from_str(json).unwrap();
            assert_eq!(result.value, expected);
        }

        assert!(serde_json::from_str::<TestStrOrIntStruct>(r#"{"value": 1.5}"#).is_err());
        assert!(serde_json::from_str::<TestStrOrIntStruct>(r#"{"value": -1}"#).is_err());
    }
}
