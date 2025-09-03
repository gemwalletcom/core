use serde::{Deserialize, Deserializer};

pub fn serialize_u128<S>(value: &u128, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_string())
}

pub fn deserialize_u128_from_str<'de, D>(deserializer: D) -> Result<u128, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<u128>().map_err(serde::de::Error::custom)
}

pub fn deserialize_option_u128_from_str<'de, D>(deserializer: D) -> Result<Option<u128>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(str_val) => str_val.parse::<u128>().map(Some).map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestStruct {
        #[serde(default, deserialize_with = "deserialize_option_u128_from_str")]
        pub amount: Option<u128>,
    }

    #[test]
    fn test_deserialize_option_u128_from_str() {
        let json = r#"{"amount": "123456789012345678901"}"#;
        let result: TestStruct = serde_json::from_str(json).unwrap();
        assert_eq!(result.amount, Some(123456789012345678901));

        let json = r#"{"amount": null}"#;
        let result: TestStruct = serde_json::from_str(json).unwrap();
        assert_eq!(result.amount, None);

        let json = r#"{}"#;
        let result: TestStruct = serde_json::from_str(json).unwrap();
        assert_eq!(result.amount, None);
    }
}
