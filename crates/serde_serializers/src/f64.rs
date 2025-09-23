use serde::{Deserialize, Deserializer};

pub fn serialize_f64<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_string())
}

pub fn deserialize_f64_from_str<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<f64>().map_err(serde::de::Error::custom)
}

pub fn deserialize_option_f64_from_str<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        Some(s) => s.parse::<f64>().map(Some).map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Serialize, de::IntoDeserializer};

    #[derive(Serialize, Deserialize)]
    struct TestOptionStruct {
        #[serde(default, deserialize_with = "deserialize_option_f64_from_str")]
        value: Option<f64>,
    }

    #[test]
    fn test_deserialize_f64_from_str() {
        let deserializer: serde::de::value::StrDeserializer<serde::de::value::Error> = "42.42".into_deserializer();
        let result = deserialize_f64_from_str(deserializer);
        assert_eq!(result.unwrap(), 42.42);

        let deserializer: serde::de::value::StrDeserializer<serde::de::value::Error> = "invalid".into_deserializer();
        let result = deserialize_f64_from_str(deserializer);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_option_f64_from_str() {
        let deserialized: TestOptionStruct = serde_json::from_str(r#"{"value":"42.42"}"#).unwrap();
        assert_eq!(deserialized.value, Some(42.42));

        let deserialized: TestOptionStruct = serde_json::from_str(r#"{}"#).unwrap();
        assert_eq!(deserialized.value, None);
    }
}
