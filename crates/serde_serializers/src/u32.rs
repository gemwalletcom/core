use std::fmt;

use serde::{Deserialize, Deserializer, de};

use crate::visitors::{StringOrNumberFromValue, StringOrNumberVisitor};

fn parse_u32_string(value: &str) -> Result<u32, String> {
    if let Some(hex_val) = value.strip_prefix("0x") {
        u32::from_str_radix(hex_val, 16).map_err(|_| format!("Invalid hex string for u32: {value}"))
    } else {
        value.parse::<u32>().map_err(|_| format!("Invalid decimal string for u32: {value}"))
    }
}

fn invalid_number(value: impl fmt::Display) -> String {
    format!("Invalid number for u32: {value}")
}

impl StringOrNumberFromValue for u32 {
    const EXPECTING: &'static str = "a string or integer representing u32";

    fn from_str(value: &str) -> Result<Self, String> {
        parse_u32_string(value)
    }

    fn from_u64(value: u64) -> Result<Self, String> {
        u32::try_from(value).map_err(|_| invalid_number(value))
    }

    fn from_i64(value: i64) -> Result<Self, String> {
        u32::try_from(value).map_err(|_| invalid_number(value))
    }
}

pub fn deserialize_u32_from_str<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    parse_u32_string(&s).map_err(de::Error::custom)
}

pub fn deserialize_u32_from_str_or_int<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(StringOrNumberVisitor::<u32>::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestStrOrInt {
        #[serde(deserialize_with = "deserialize_u32_from_str_or_int")]
        pub value: u32,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestStr {
        #[serde(deserialize_with = "deserialize_u32_from_str")]
        pub value: u32,
    }

    #[test]
    fn test_u32_deserialization() {
        let mixed_cases = [
            (r#"{"value": 42}"#, 42u32),
            (r#"{"value": "42"}"#, 42),
            (r#"{"value": "0x2a"}"#, 42),
            (r#"{"value": 0}"#, 0),
            (r#"{"value": "4294967295"}"#, u32::MAX),
        ];
        for (json, expected) in mixed_cases {
            let result: TestStrOrInt = serde_json::from_str(json).unwrap();
            assert_eq!(result.value, expected);
        }

        let str_cases = [
            (r#"{"value": "0"}"#, 0u32),
            (r#"{"value": "100"}"#, 100),
            (r#"{"value": "0xff"}"#, 255),
        ];
        for (json, expected) in str_cases {
            let result: TestStr = serde_json::from_str(json).unwrap();
            assert_eq!(result.value, expected);
        }

        assert!(serde_json::from_str::<TestStrOrInt>(r#"{"value": 4294967296}"#).is_err());
        assert!(serde_json::from_str::<TestStrOrInt>(r#"{"value": -1}"#).is_err());
        assert!(serde_json::from_str::<TestStrOrInt>(r#"{"value": 1.5}"#).is_err());
    }
}
