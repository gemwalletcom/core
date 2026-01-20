use std::fmt;

use serde::Deserializer;

use crate::visitors::{NumberFromValue, OptionNumberVisitor};

fn out_of_range(value: impl fmt::Display) -> String {
    format!("number {} is out of range for u32", value)
}

impl NumberFromValue for u32 {
    fn from_u64(value: u64) -> Result<Self, String> {
        if value > u32::MAX as u64 {
            return Err(out_of_range(value));
        }
        Ok(value as u32)
    }

    fn from_i64(value: i64) -> Result<Self, String> {
        if value < 0 || value > u32::MAX as i64 {
            return Err(out_of_range(value));
        }
        Ok(value as u32)
    }
}

/// Deserialize an Option<u32> from a number (integer only).
pub fn deserialize_option_u32_from_number<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(OptionNumberVisitor::<u32>::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestStruct {
        #[serde(default, deserialize_with = "deserialize_option_u32_from_number")]
        pub time_estimate: Option<u32>,
    }

    #[test]
    fn test_deserialize_option_u32_from_number() {
        // Integer
        let result: TestStruct = serde_json::from_str(r#"{"time_estimate": 10}"#).unwrap();
        assert_eq!(result.time_estimate, Some(10));

        // Null
        let result: TestStruct = serde_json::from_str(r#"{"time_estimate": null}"#).unwrap();
        assert_eq!(result.time_estimate, None);

        // Missing field
        let result: TestStruct = serde_json::from_str(r#"{}"#).unwrap();
        assert_eq!(result.time_estimate, None);

        // Zero
        let result: TestStruct = serde_json::from_str(r#"{"time_estimate": 0}"#).unwrap();
        assert_eq!(result.time_estimate, Some(0));
    }

    #[test]
    fn test_deserialize_option_u32_from_number_rejects_invalid_types() {
        let result: Result<TestStruct, _> = serde_json::from_str(r#"{"time_estimate": "10"}"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_option_u32_from_number_rejects_out_of_range() {
        let result: Result<TestStruct, _> = serde_json::from_str(r#"{"time_estimate": 4294967296}"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_option_u32_from_number_rejects_float() {
        let result: Result<TestStruct, _> = serde_json::from_str(r#"{"time_estimate": 7.4}"#);
        assert!(result.is_err());
    }
}
