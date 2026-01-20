use serde::Deserializer;

use crate::visitors::StringFromValueVisitor;

/// Deserialize a String from either a string, number, or null JSON value
pub fn deserialize_string_from_value<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(StringFromValueVisitor::new(true))
}

/// Deserialize a String from either a string or a number JSON value
pub fn deserialize_string_from_str_or_number<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(StringFromValueVisitor::new(false))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestStruct {
        #[serde(deserialize_with = "deserialize_string_from_str_or_number")]
        pub value: String,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestStructWithNull {
        #[serde(default, deserialize_with = "deserialize_string_from_value")]
        pub value: String,
    }

    #[test]
    fn test_deserialize_string_from_str_or_number() {
        // String
        let result: TestStruct = serde_json::from_str(r#"{"value": "hello"}"#).unwrap();
        assert_eq!(result.value, "hello");

        // Integer
        let result: TestStruct = serde_json::from_str(r#"{"value": 123}"#).unwrap();
        assert_eq!(result.value, "123");

        // Zero
        let result: TestStruct = serde_json::from_str(r#"{"value": 0}"#).unwrap();
        assert_eq!(result.value, "0");

        // Float
        let result: TestStruct = serde_json::from_str(r#"{"value": 123.456}"#).unwrap();
        assert_eq!(result.value, "123.456");
    }

    #[test]
    fn test_deserialize_string_from_value() {
        // String
        let result: TestStructWithNull = serde_json::from_str(r#"{"value": "hello"}"#).unwrap();
        assert_eq!(result.value, "hello");

        // Integer
        let result: TestStructWithNull = serde_json::from_str(r#"{"value": 123}"#).unwrap();
        assert_eq!(result.value, "123");

        // Zero
        let result: TestStructWithNull = serde_json::from_str(r#"{"value": 0}"#).unwrap();
        assert_eq!(result.value, "0");

        // Null
        let result: TestStructWithNull = serde_json::from_str(r#"{"value": null}"#).unwrap();
        assert_eq!(result.value, "");

        // Float
        let result: TestStructWithNull = serde_json::from_str(r#"{"value": 123.456}"#).unwrap();
        assert_eq!(result.value, "123.456");
    }
}
