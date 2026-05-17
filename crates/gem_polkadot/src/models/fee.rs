use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_biguint_from_str;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolkadotEstimateFee {
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub partial_fee: BigUint,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_fee_deserializes_base_units_without_narrowing() {
        let fee: PolkadotEstimateFee = serde_json::from_str(r#"{"partialFee":"18446744073709551616"}"#).unwrap();

        assert_eq!(fee.partial_fee, BigUint::parse_bytes(b"18446744073709551616", 10).unwrap());
    }
}
