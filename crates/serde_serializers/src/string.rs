use serde::Deserializer;

use crate::visitors::StringFromValueVisitor;

pub fn deserialize_string_from_value<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(StringFromValueVisitor::new(true))
}

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
    fn test_string_deserialization() {
        let str_or_number_cases = [
            (r#"{"value": "hello"}"#, "hello"),
            (r#"{"value": 123}"#, "123"),
            (r#"{"value": 0}"#, "0"),
            (r#"{"value": 123.456}"#, "123.456"),
        ];
        for (json, expected) in str_or_number_cases {
            let result: TestStruct = serde_json::from_str(json).unwrap();
            assert_eq!(result.value, expected);
        }

        let value_cases = [
            (r#"{"value": "hello"}"#, "hello"),
            (r#"{"value": 123}"#, "123"),
            (r#"{"value": 0}"#, "0"),
            (r#"{"value": null}"#, ""),
            (r#"{"value": 123.456}"#, "123.456"),
        ];
        for (json, expected) in value_cases {
            let result: TestStructWithNull = serde_json::from_str(json).unwrap();
            assert_eq!(result.value, expected);
        }
    }
}
