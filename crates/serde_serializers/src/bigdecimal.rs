use bigdecimal::BigDecimal;
use serde::{Deserialize, Deserializer};

/// Custom deserializer for BigDecimal from f64 that preserves full precision
pub fn deserialize_bigdecimal_from_f64<'de, D>(deserializer: D) -> Result<BigDecimal, D::Error>
where
    D: Deserializer<'de>,
{
    let value = f64::deserialize(deserializer)?;
    // Use the full precision from f64 by converting to string first
    let value_str = format!("{value}");
    value_str.parse::<BigDecimal>()
        .map_err(serde::de::Error::custom)
}

/// Custom deserializer for Option<BigDecimal> from f64 that preserves full precision
pub fn deserialize_option_bigdecimal_from_f64<'de, D>(deserializer: D) -> Result<Option<BigDecimal>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<f64>::deserialize(deserializer)?;
    match value {
        Some(v) => {
            let value_str = format!("{v}");
            value_str.parse::<BigDecimal>()
                .map(Some)
                .map_err(serde::de::Error::custom)
        }
        None => Ok(None),
    }
}