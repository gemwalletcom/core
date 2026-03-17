use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
pub struct Long {
    pub low: i32,
    pub high: i32,
}

impl Long {
    pub fn to_uint64(&self) -> u64 {
        ((self.high as u32 as u64) << 32) | (self.low as u32 as u64)
    }
}

pub fn deserialize_u64_from_long_or_int<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum LongOrValue {
        Number(u64),
        Str(String),
        Long(Long),
    }

    match LongOrValue::deserialize(deserializer)? {
        LongOrValue::Number(n) => Ok(n),
        LongOrValue::Str(s) => s.parse::<u64>().map_err(serde::de::Error::custom),
        LongOrValue::Long(l) => Ok(l.to_uint64()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_uint64() {
        let l = Long { low: -72998656, high: 412955876 };
        assert_eq!(l.to_uint64(), 1773631986332999936);
    }

    #[test]
    fn test_to_uint64_small() {
        let l = Long { low: 1, high: 0 };
        assert_eq!(l.to_uint64(), 1);
    }

    #[derive(Deserialize)]
    struct TestTimestamp {
        #[serde(deserialize_with = "deserialize_u64_from_long_or_int")]
        ts: u64,
    }

    #[test]
    fn test_deserialize_number() {
        let v: TestTimestamp = serde_json::from_str(r#"{"ts": 1773382733549000000}"#).unwrap();
        assert_eq!(v.ts, 1773382733549000000);
    }

    #[test]
    fn test_deserialize_string() {
        let v: TestTimestamp = serde_json::from_str(r#"{"ts": "1773382733549000000"}"#).unwrap();
        assert_eq!(v.ts, 1773382733549000000);
    }

    #[test]
    fn test_deserialize_long() {
        let v: TestTimestamp = serde_json::from_str(r#"{"ts": {"low": -72998656, "high": 412955876, "unsigned": false}}"#).unwrap();
        assert_eq!(v.ts, 1773631986332999936);
    }
}
