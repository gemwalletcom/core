use num_bigint::BigInt;
use serde::{Deserialize, de};

fn parse_bigint_hex(value: &str) -> Result<BigInt, String> {
    let hex_value = value.strip_prefix("0x").unwrap_or(value);
    if hex_value.is_empty() {
        return Ok(BigInt::from(0));
    }
    BigInt::parse_bytes(hex_value.as_bytes(), 16).ok_or_else(|| format!("Invalid hex string: {value}"))
}

fn parse_bigint_str(value: &str) -> Result<BigInt, String> {
    match value.strip_prefix("0x") {
        Some(_) => parse_bigint_hex(value),
        None => value.parse::<BigInt>().map_err(|err| err.to_string()),
    }
}

pub fn serialize_bigint<S>(value: &BigInt, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_string())
}

pub fn deserialize_bigint_from_str<'de, D>(deserializer: D) -> Result<BigInt, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: String = de::Deserialize::deserialize(deserializer)?;
    parse_bigint_str(&s).map_err(de::Error::custom)
}

pub fn deserialize_option_bigint_from_str<'de, D>(deserializer: D) -> Result<Option<BigInt>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(str_val) => parse_bigint_str(&str_val).map(Some).map_err(de::Error::custom),
        None => Ok(None),
    }
}

pub fn bigint_from_hex_str(hex_str: &str) -> Result<BigInt, Box<dyn std::error::Error + Send + Sync>> {
    parse_bigint_hex(hex_str).map_err(|err| err.into())
}

pub fn deserialize_bigint_vec_from_hex_str<'de, D>(deserializer: D) -> Result<Vec<BigInt>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let hex_strings: Vec<String> = de::Deserialize::deserialize(deserializer)?;
    hex_strings.into_iter().map(|hex_str| bigint_from_hex_str(&hex_str).map_err(de::Error::custom)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigInt;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct TestStruct {
        #[serde(serialize_with = "serialize_bigint", deserialize_with = "deserialize_bigint_from_str")]
        value: BigInt,
    }

    #[derive(Deserialize)]
    struct TestVecStruct {
        #[serde(deserialize_with = "deserialize_bigint_vec_from_hex_str")]
        values: Vec<BigInt>,
    }

    #[test]
    fn test_bigint_serialization() {
        let value = BigInt::parse_bytes(b"12345678901234567890", 10).unwrap();
        let test_struct = TestStruct { value };
        let serialized = serde_json::to_string(&test_struct).unwrap();
        assert_eq!(serialized, r#"{"value":"12345678901234567890"}"#);

        let deserialized: TestStruct = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.value, BigInt::parse_bytes(b"12345678901234567890", 10).unwrap());

        let hex_cases = [
            (r#"{"value":"0xff"}"#, BigInt::from(255)),
            (r#"{"value":"0x0"}"#, BigInt::from(0)),
            (r#"{"value":"0x"}"#, BigInt::from(0)),
        ];
        for (json, expected) in hex_cases {
            let deserialized: TestStruct = serde_json::from_str(json).unwrap();
            assert_eq!(deserialized.value, expected);
        }
    }

    #[test]
    fn test_bigint_from_hex_str() {
        assert_eq!(bigint_from_hex_str("0x1a").unwrap(), BigInt::from(26));
        assert_eq!(bigint_from_hex_str("1a").unwrap(), BigInt::from(26));
        assert_eq!(bigint_from_hex_str("0x0").unwrap(), BigInt::from(0));
        assert_eq!(bigint_from_hex_str("0x").unwrap(), BigInt::from(0));
        assert_eq!(bigint_from_hex_str("ff").unwrap(), BigInt::from(255));
        assert!(bigint_from_hex_str("xyz").is_err());

        let deserialized: TestVecStruct = serde_json::from_str(r#"{"values":["0x1a","0xff","0x0"]}"#).unwrap();
        assert_eq!(deserialized.values, vec![BigInt::from(26), BigInt::from(255), BigInt::from(0)]);
    }
}
