use serde::{Deserialize, Deserializer};
use serde_json::Value;

/// Deserialize an Option<u32> from a number (integer or float, rounded)
pub fn deserialize_option_u32_from_number<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    match value {
        Some(Value::Number(n)) => {
            if let Some(i) = n.as_u64() {
                Ok(Some(i as u32))
            } else if let Some(f) = n.as_f64() {
                Ok(Some(f.round() as u32))
            } else {
                Ok(None)
            }
        }
        _ => Ok(None),
    }
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

        // Float rounds down
        let result: TestStruct = serde_json::from_str(r#"{"time_estimate": 7.4}"#).unwrap();
        assert_eq!(result.time_estimate, Some(7));

        // Float rounds up
        let result: TestStruct = serde_json::from_str(r#"{"time_estimate": 7.5}"#).unwrap();
        assert_eq!(result.time_estimate, Some(8));

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
}
